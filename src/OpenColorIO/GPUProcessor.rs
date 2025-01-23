use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct GPUProcessor {
    impl_: Arc<Mutex<Impl>>,
}

#[derive(Clone, Debug)]
struct Impl {
    ops: Vec<Box<dyn Op>>,
    is_no_op: bool,
    has_channel_crosstalk: bool,
    cache_id: String,
}

impl Impl {
    fn new() -> Self {
        Impl {
            ops: Vec::new(),
            is_no_op: false,
            has_channel_crosstalk: true,
            cache_id: String::new(),
        }
    }

    fn finalize(&mut self, raw_ops: &OpRcPtrVec, optimization_flags: OptimizationFlags) {
        let mut ops = raw_ops.clone();
        ops.finalize();
        ops.optimize(optimization_flags);
        ops.validate_dynamic_properties();

        self.is_no_op = ops.is_no_op();
        self.has_channel_crosstalk = ops.has_channel_crosstalk();

        let mut ss = String::new();
        ss.push_str(&format!("GPU Processor: oFlags {:?} ops: {}", optimization_flags, ops.get_cache_id()));
        self.cache_id = ss;
    }

    fn extract_gpu_shader_info(&self, shader_creator: &mut GpuShaderCreator) {
        for op in &self.ops {
            op.extract_gpu_shader_info(shader_creator);
        }

        write_shader_header(shader_creator);
        write_shader_footer(shader_creator);

        shader_creator.finalize();
    }
}

impl GPUProcessor {
    pub fn new() -> Self {
        GPUProcessor {
            impl_: Arc::new(Mutex::new(Impl::new())),
        }
    }

    pub fn is_no_op(&self) -> bool {
        self.impl_.lock().unwrap().is_no_op
    }

    pub fn has_channel_crosstalk(&self) -> bool {
        self.impl_.lock().unwrap().has_channel_crosstalk
    }

    pub fn get_cache_id(&self) -> String {
        self.impl_.lock().unwrap().cache_id.clone()
    }

    pub fn extract_gpu_shader_info(&self, shader_desc: &mut GpuShaderDesc) {
        let mut shader_creator = GpuShaderCreator::from(shader_desc);
        self.impl_.lock().unwrap().extract_gpu_shader_info(&mut shader_creator);
    }

    pub fn extract_gpu_shader_info_with_creator(&self, shader_creator: &mut GpuShaderCreator) {
        let mut key = shader_creator.get_cache_id().to_string();
        key.push_str(&self.impl_.lock().unwrap().cache_id);

        let key = cache_id_hash(&key);

        if !key.chars().next().unwrap().is_alphabetic() {
            key.insert(0, 'k');
        }

        let key = key.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect::<String>();

        shader_creator.begin(&key);

        self.impl_.lock().unwrap().extract_gpu_shader_info(shader_creator);

        shader_creator.end();
    }
}

fn write_shader_header(shader_creator: &mut GpuShaderCreator) {
    let fcn_name = shader_creator.get_function_name().to_string();

    let mut ss = GpuShaderText::new(shader_creator.get_language());

    ss.new_line();
    ss.new_line().push_str("// Declaration of the OCIO shader function");
    ss.new_line();

    if shader_creator.get_language() == GpuLanguage::OSL_1 {
        ss.new_line().push_str(&format!("color4 {}(color4 inPixel)", fcn_name));
        ss.new_line().push_str("{");
        ss.indent();
        ss.new_line().push_str(&format!("color4 {} = inPixel;", shader_creator.get_pixel_name()));
    } else {
        ss.new_line().push_str(&format!("{} {}({} inPixel)", ss.float4_keyword(), fcn_name, ss.float4_keyword()));
        ss.new_line().push_str("{");
        ss.indent();
        ss.new_line().push_str(&format!("{} {} = inPixel;", ss.float4_decl(), shader_creator.get_pixel_name()));
    }

    shader_creator.add_to_function_header_shader_code(&ss.string());
}

fn write_shader_footer(shader_creator: &mut GpuShaderCreator) {
    let mut ss = GpuShaderText::new(shader_creator.get_language());

    ss.new_line();
    ss.indent();
    ss.new_line().push_str(&format!("return {};", shader_creator.get_pixel_name()));
    ss.dedent();
    ss.new_line().push_str("}");

    shader_creator.add_to_function_footer_shader_code(&ss.string());
}

fn cache_id_hash(key: &str) -> String {
    let hash = xxhash_rust::xxh3::xxh3_128(key.as_bytes());

    format!("{:x}{:x}", hash.low64, hash.high64)
}
