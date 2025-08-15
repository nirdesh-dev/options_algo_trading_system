use super::params::IronCondorParams;
use super::result::IronCondorBacktestResult;
use crate::domain::PricingModel;
use crate::strategy::iron_condor::config::IronCondorConfig;
use anyhow::Result;
use cudarc::{
    driver::{CudaContext, LaunchConfig, PushKernelArg},
    nvrtc::{compile_ptx_with_opts, CompileOptions},
};
use include_dir::{include_dir, Dir};
pub struct IronCondor {
    config: IronCondorConfig,
}

impl IronCondor {
    pub fn new(config: IronCondorConfig) -> Self {
        Self { config }
    }

    fn generate_param_grid(&self) -> Vec<IronCondorParams> {
        self.config.generate_param_grid()
    }

    pub fn run_backtest_on_gpu(
        &self,
        price_series: &[f32],
        pricing_model: PricingModel,
    ) -> Result<Vec<IronCondorBacktestResult>> {
        static KERNELS: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/cuda/kernels");

        let opts = CompileOptions {
            include_paths: vec![
                "/usr/include".into(),
                "/usr/include/x86_64-linux-gnu".into(),
                "src/cuda/kernels".into(),
            ],
            ..Default::default()
        };

        // 1. Generate the param grid
        let grid = self.generate_param_grid();

        // 2. Convert params to f32 slices for kernel inputs
        let entry_times: Vec<f32> = grid.iter().map(|p| p.entry_time.value() as f32).collect();
        let wing_widths: Vec<f32> = grid.iter().map(|p| p.wing_width.value()).collect();
        let stop_losses: Vec<f32> = grid.iter().map(|p| p.stop_loss.value()).collect();

        let num_params = grid.len();

        // 3. Create CUDA context and stream
        let device = CudaContext::new(0)?;
        let stream = device.default_stream();

        // 4. Copy inputs to GPU device memory
        let prices_d = stream.memcpy_stod(price_series)?;
        let entry_times_d = stream.memcpy_stod(&entry_times)?;
        let wing_widths_d = stream.memcpy_stod(&wing_widths)?;
        let stop_losses_d = stream.memcpy_stod(&stop_losses)?;

        // 5. Allocate output buffers on device
        let mut pnl_d = stream.alloc_zeros::<f32>(num_params)?;
        let mut drawdown_d = stream.alloc_zeros::<f32>(num_params)?;
        let mut sharpe_d = stream.alloc_zeros::<f32>(num_params)?;

        // 6. Compile/load CUDA kernel
        // Make sure your kernel file path is correct

        let lib_cuh = KERNELS
            .get_file("lib.cuh")
            .expect("lib.cuh not found")
            .contents_utf8()
            .expect("lib.cuh invalid utf8");

        let iron_condor_cu = KERNELS
            .get_file("iron_condor.cu")
            .expect("iron_condor.cu not found")
            .contents_utf8()
            .expect("iron_condor.cu invalid utf8");

        let full_src = format!("{}\n{}", lib_cuh, iron_condor_cu);

        let ptx = compile_ptx_with_opts(&full_src, opts)?;
        let module = device.load_module(ptx)?;
        let kernel = module.load_function("iron_condor_kernel")?;

        // 7. Prepare kernel launch
        let mut builder = stream.launch_builder(&kernel);

        let cfg = LaunchConfig::for_num_elems(num_params as u32);

        let price_series_len = price_series.len() as i32;
        let num_params_i32 = num_params as i32;
        let pricing_flag = pricing_model.to_kernel_flag();

        builder
            .arg(&prices_d)
            .arg(&(price_series_len))
            .arg(&entry_times_d)
            .arg(&wing_widths_d)
            .arg(&stop_losses_d)
            .arg(&(num_params_i32))
            .arg(&pricing_flag)
            .arg(&mut pnl_d)
            .arg(&mut drawdown_d)
            .arg(&mut sharpe_d);

        // 8. Launch the kernel
        unsafe {
            builder.launch(cfg)?;
        }

        // 9. Retrieve results from device to host
        let pnl = stream.memcpy_dtov(&pnl_d)?;
        let drawdown = stream.memcpy_dtov(&drawdown_d)?;
        let sharpe = stream.memcpy_dtov(&sharpe_d)?;

        // 10. Build results struct
        let results = grid
            .into_iter()
            .zip(pnl.into_iter().zip(drawdown).zip(sharpe))
            .map(|(params, ((pnl, dd), sh))| IronCondorBacktestResult {
                params,
                total_pnl: pnl,
                drawdown: dd,
                sharpe: sh,
            })
            .collect();

        Ok(results)
    }
}
