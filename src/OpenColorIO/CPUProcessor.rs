use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct CPUProcessor {
    impl_: Arc<Mutex<Impl>>,
}

#[derive(Clone, Debug)]
struct Impl {
    in_bit_depth_op: Option<Box<dyn OpCPU>>,
    cpu_ops: Vec<Box<dyn OpCPU>>,
    out_bit_depth_op: Option<Box<dyn OpCPU>>,
    in_bit_depth: BitDepth,
    out_bit_depth: BitDepth,
    is_no_op: bool,
    is_identity: bool,
    has_channel_crosstalk: bool,
    cache_id: String,
}

impl Impl {
    fn new() -> Self {
        Impl {
            in_bit_depth_op: None,
            cpu_ops: Vec::new(),
            out_bit_depth_op: None,
            in_bit_depth: BitDepth::F32,
            out_bit_depth: BitDepth::F32,
            is_no_op: false,
            is_identity: false,
            has_channel_crosstalk: true,
            cache_id: String::new(),
        }
    }

    fn is_dynamic(&self) -> bool {
        if let Some(ref op) = self.in_bit_depth_op {
            if op.is_dynamic() {
                return true;
            }
        }

        for op in &self.cpu_ops {
            if op.is_dynamic() {
                return true;
            }
        }

        if let Some(ref op) = self.out_bit_depth_op {
            if op.is_dynamic() {
                return true;
            }
        }

        false
    }

    fn has_dynamic_property(&self, property_type: DynamicPropertyType) -> bool {
        if let Some(ref op) = self.in_bit_depth_op {
            if op.has_dynamic_property(property_type) {
                return true;
            }
        }

        for op in &self.cpu_ops {
            if op.has_dynamic_property(property_type) {
                return true;
            }
        }

        if let Some(ref op) = self.out_bit_depth_op {
            if op.has_dynamic_property(property_type) {
                return true;
            }
        }

        false
    }

    fn get_dynamic_property(&self, property_type: DynamicPropertyType) -> DynamicPropertyRcPtr {
        if let Some(ref op) = self.in_bit_depth_op {
            if op.has_dynamic_property(property_type) {
                return op.get_dynamic_property(property_type);
            }
        }

        for op in &self.cpu_ops {
            if op.has_dynamic_property(property_type) {
                return op.get_dynamic_property(property_type);
            }
        }

        if let Some(ref op) = self.out_bit_depth_op {
            if op.has_dynamic_property(property_type) {
                return op.get_dynamic_property(property_type);
            }
        }

        panic!("Cannot find dynamic property; not used by CPU processor.");
    }

    fn finalize(&mut self, raw_ops: &OpRcPtrVec, in_bit_depth: BitDepth, out_bit_depth: BitDepth, optimization_flags: OptimizationFlags) {
        let mut ops = raw_ops.clone();
        ops.finalize();
        ops.optimize(optimization_flags);
        ops.optimize_for_bitdepth(in_bit_depth, out_bit_depth, optimization_flags);

        self.in_bit_depth = in_bit_depth;
        self.out_bit_depth = out_bit_depth;

        self.is_identity = ops.is_no_op();
        self.is_no_op = self.is_identity && self.in_bit_depth == self.out_bit_depth;

        self.has_channel_crosstalk = ops.has_channel_crosstalk();

        self.cpu_ops.clear();
        self.in_bit_depth_op = None;
        self.out_bit_depth_op = None;
        create_cpu_engine(&ops, in_bit_depth, out_bit_depth, optimization_flags, &mut self.in_bit_depth_op, &mut self.cpu_ops, &mut self.out_bit_depth_op);

        let mut ss = String::new();
        ss.push_str(&format!("CPU Processor: from {:?} to {:?} oFlags {:?} ops: {}", in_bit_depth, out_bit_depth, optimization_flags, ops.get_cache_id()));
        self.cache_id = ss;
    }

    fn apply(&self, img_desc: &ImageDesc) {
        let mut scanline_builder = create_scanline_helper(self.in_bit_depth, &self.in_bit_depth_op, self.out_bit_depth, &self.out_bit_depth_op);
        scanline_builder.init(img_desc);

        let mut rgba_buffer: Option<&mut [f32]> = None;
        let mut num_pixels = 0;

        while {
            scanline_builder.prep_rgba_scanline(&mut rgba_buffer, &mut num_pixels);
            num_pixels > 0
        } {
            for op in &self.cpu_ops {
                op.apply(rgba_buffer.as_mut().unwrap(), rgba_buffer.as_mut().unwrap(), num_pixels);
            }

            scanline_builder.finish_rgba_scanline();
        }
    }

    fn apply_with_dst(&self, src_img_desc: &ImageDesc, dst_img_desc: &mut ImageDesc) {
        let mut scanline_builder = create_scanline_helper(self.in_bit_depth, &self.in_bit_depth_op, self.out_bit_depth, &self.out_bit_depth_op);
        scanline_builder.init_with_dst(src_img_desc, dst_img_desc);

        let mut rgba_buffer: Option<&mut [f32]> = None;
        let mut num_pixels = 0;

        while {
            scanline_builder.prep_rgba_scanline(&mut rgba_buffer, &mut num_pixels);
            num_pixels > 0
        } {
            for op in &self.cpu_ops {
                op.apply(rgba_buffer.as_mut().unwrap(), rgba_buffer.as_mut().unwrap(), num_pixels);
            }

            scanline_builder.finish_rgba_scanline();
        }
    }

    fn apply_rgb(&self, pixel: &mut [f32; 3]) {
        let mut v = [pixel[0], pixel[1], pixel[2], 0.0];

        if let Some(ref op) = self.in_bit_depth_op {
            op.apply(&mut v, &mut v, 1);
        }

        for op in &self.cpu_ops {
            op.apply(&mut v, &mut v, 1);
        }

        if let Some(ref op) = self.out_bit_depth_op {
            op.apply(&mut v, &mut v, 1);
        }

        pixel[0] = v[0];
        pixel[1] = v[1];
        pixel[2] = v[2];
    }

    fn apply_rgba(&self, pixel: &mut [f32; 4]) {
        if let Some(ref op) = self.in_bit_depth_op {
            op.apply(pixel, pixel, 1);
        }

        for op in &self.cpu_ops {
            op.apply(pixel, pixel, 1);
        }

        if let Some(ref op) = self.out_bit_depth_op {
            op.apply(pixel, pixel, 1);
        }
    }
}

impl CPUProcessor {
    pub fn new() -> Self {
        CPUProcessor {
            impl_: Arc::new(Mutex::new(Impl::new())),
        }
    }

    pub fn is_no_op(&self) -> bool {
        self.impl_.lock().unwrap().is_no_op
    }

    pub fn is_identity(&self) -> bool {
        self.impl_.lock().unwrap().is_identity
    }

    pub fn has_channel_crosstalk(&self) -> bool {
        self.impl_.lock().unwrap().has_channel_crosstalk
    }

    pub fn get_cache_id(&self) -> String {
        self.impl_.lock().unwrap().cache_id.clone()
    }

    pub fn get_input_bit_depth(&self) -> BitDepth {
        self.impl_.lock().unwrap().in_bit_depth
    }

    pub fn get_output_bit_depth(&self) -> BitDepth {
        self.impl_.lock().unwrap().out_bit_depth
    }

    pub fn is_dynamic(&self) -> bool {
        self.impl_.lock().unwrap().is_dynamic()
    }

    pub fn has_dynamic_property(&self, property_type: DynamicPropertyType) -> bool {
        self.impl_.lock().unwrap().has_dynamic_property(property_type)
    }

    pub fn get_dynamic_property(&self, property_type: DynamicPropertyType) -> DynamicPropertyRcPtr {
        self.impl_.lock().unwrap().get_dynamic_property(property_type)
    }

    pub fn apply(&self, img_desc: &ImageDesc) {
        self.impl_.lock().unwrap().apply(img_desc)
    }

    pub fn apply_with_dst(&self, src_img_desc: &ImageDesc, dst_img_desc: &mut ImageDesc) {
        self.impl_.lock().unwrap().apply_with_dst(src_img_desc, dst_img_desc)
    }

    pub fn apply_rgb(&self, pixel: &mut [f32; 3]) {
        self.impl_.lock().unwrap().apply_rgb(pixel)
    }

    pub fn apply_rgba(&self, pixel: &mut [f32; 4]) {
        self.impl_.lock().unwrap().apply_rgba(pixel)
    }
}
