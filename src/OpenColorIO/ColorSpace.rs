use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct ColorSpace {
    impl_: Arc<Mutex<Impl>>,
}

#[derive(Clone, Debug)]
struct Impl {
    name: String,
    family: String,
    equality_group: String,
    description: String,
    encoding: String,
    aliases: Vec<String>,
    bit_depth: BitDepth,
    is_data: bool,
    reference_space_type: ReferenceSpaceType,
    allocation: Allocation,
    allocation_vars: Vec<f32>,
    to_ref_transform: Option<TransformRcPtr>,
    from_ref_transform: Option<TransformRcPtr>,
    to_ref_specified: bool,
    from_ref_specified: bool,
    categories: TokensManager,
}

impl Impl {
    fn new(reference_space: ReferenceSpaceType) -> Self {
        Impl {
            name: String::new(),
            family: String::new(),
            equality_group: String::new(),
            description: String::new(),
            encoding: String::new(),
            aliases: Vec::new(),
            bit_depth: BitDepth::Unknown,
            is_data: false,
            reference_space_type: reference_space,
            allocation: Allocation::Uniform,
            allocation_vars: Vec::new(),
            to_ref_transform: None,
            from_ref_transform: None,
            to_ref_specified: false,
            from_ref_specified: false,
            categories: TokensManager::new(),
        }
    }
}

impl ColorSpace {
    pub fn create() -> ColorSpaceRcPtr {
        Arc::new(ColorSpace {
            impl_: Arc::new(Mutex::new(Impl::new(ReferenceSpaceType::Scene))),
        })
    }

    pub fn create_with_reference_space(reference_space: ReferenceSpaceType) -> ColorSpaceRcPtr {
        Arc::new(ColorSpace {
            impl_: Arc::new(Mutex::new(Impl::new(reference_space))),
        })
    }

    pub fn create_editable_copy(&self) -> ColorSpaceRcPtr {
        let impl_copy = self.impl_.lock().unwrap().clone();
        Arc::new(ColorSpace {
            impl_: Arc::new(Mutex::new(impl_copy)),
        })
    }

    pub fn get_name(&self) -> String {
        self.impl_.lock().unwrap().name.clone()
    }

    pub fn set_name(&self, name: &str) {
        let mut impl_ = self.impl_.lock().unwrap();
        impl_.name = name.to_string();
        impl_.aliases.retain(|alias| alias != name);
    }

    pub fn get_num_aliases(&self) -> usize {
        self.impl_.lock().unwrap().aliases.len()
    }

    pub fn get_alias(&self, idx: usize) -> String {
        self.impl_.lock().unwrap().aliases.get(idx).cloned().unwrap_or_default()
    }

    pub fn has_alias(&self, alias: &str) -> bool {
        self.impl_.lock().unwrap().aliases.iter().any(|a| a.eq_ignore_ascii_case(alias))
    }

    pub fn add_alias(&self, alias: &str) {
        let mut impl_ = self.impl_.lock().unwrap();
        if alias != impl_.name && !impl_.aliases.contains(&alias.to_string()) {
            impl_.aliases.push(alias.to_string());
        }
    }

    pub fn remove_alias(&self, alias: &str) {
        let mut impl_ = self.impl_.lock().unwrap();
        impl_.aliases.retain(|a| a != alias);
    }

    pub fn clear_aliases(&self) {
        self.impl_.lock().unwrap().aliases.clear();
    }

    pub fn get_family(&self) -> String {
        self.impl_.lock().unwrap().family.clone()
    }

    pub fn set_family(&self, family: &str) {
        self.impl_.lock().unwrap().family = family.to_string();
    }

    pub fn get_equality_group(&self) -> String {
        self.impl_.lock().unwrap().equality_group.clone()
    }

    pub fn set_equality_group(&self, equality_group: &str) {
        self.impl_.lock().unwrap().equality_group = equality_group.to_string();
    }

    pub fn get_description(&self) -> String {
        self.impl_.lock().unwrap().description.clone()
    }

    pub fn set_description(&self, description: &str) {
        self.impl_.lock().unwrap().description = description.to_string();
    }

    pub fn get_bit_depth(&self) -> BitDepth {
        self.impl_.lock().unwrap().bit_depth
    }

    pub fn set_bit_depth(&self, bit_depth: BitDepth) {
        self.impl_.lock().unwrap().bit_depth = bit_depth;
    }

    pub fn has_category(&self, category: &str) -> bool {
        self.impl_.lock().unwrap().categories.has_token(category)
    }

    pub fn add_category(&self, category: &str) {
        self.impl_.lock().unwrap().categories.add_token(category);
    }

    pub fn remove_category(&self, category: &str) {
        self.impl_.lock().unwrap().categories.remove_token(category);
    }

    pub fn get_num_categories(&self) -> usize {
        self.impl_.lock().unwrap().categories.get_num_tokens()
    }

    pub fn get_category(&self, index: usize) -> String {
        self.impl_.lock().unwrap().categories.get_token(index).to_string()
    }

    pub fn clear_categories(&self) {
        self.impl_.lock().unwrap().categories.clear_tokens();
    }

    pub fn get_encoding(&self) -> String {
        self.impl_.lock().unwrap().encoding.clone()
    }

    pub fn set_encoding(&self, encoding: &str) {
        self.impl_.lock().unwrap().encoding = encoding.to_string();
    }

    pub fn is_data(&self) -> bool {
        self.impl_.lock().unwrap().is_data
    }

    pub fn set_is_data(&self, val: bool) {
        self.impl_.lock().unwrap().is_data = val;
    }

    pub fn get_reference_space_type(&self) -> ReferenceSpaceType {
        self.impl_.lock().unwrap().reference_space_type
    }

    pub fn get_allocation(&self) -> Allocation {
        self.impl_.lock().unwrap().allocation
    }

    pub fn set_allocation(&self, allocation: Allocation) {
        self.impl_.lock().unwrap().allocation = allocation;
    }

    pub fn get_allocation_num_vars(&self) -> usize {
        self.impl_.lock().unwrap().allocation_vars.len()
    }

    pub fn get_allocation_vars(&self) -> Vec<f32> {
        self.impl_.lock().unwrap().allocation_vars.clone()
    }

    pub fn set_allocation_vars(&self, vars: &[f32]) {
        self.impl_.lock().unwrap().allocation_vars = vars.to_vec();
    }

    pub fn get_transform(&self, dir: ColorSpaceDirection) -> Option<TransformRcPtr> {
        match dir {
            ColorSpaceDirection::ToReference => self.impl_.lock().unwrap().to_ref_transform.clone(),
            ColorSpaceDirection::FromReference => self.impl_.lock().unwrap().from_ref_transform.clone(),
        }
    }

    pub fn set_transform(&self, transform: Option<TransformRcPtr>, dir: ColorSpaceDirection) {
        let mut impl_ = self.impl_.lock().unwrap();
        match dir {
            ColorSpaceDirection::ToReference => impl_.to_ref_transform = transform,
            ColorSpaceDirection::FromReference => impl_.from_ref_transform = transform,
        }
    }
}

impl std::fmt::Display for ColorSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let impl_ = self.impl_.lock().unwrap();
        let num_vars = impl_.allocation_vars.len();
        let vars: Vec<String> = impl_.allocation_vars.iter().map(|v| v.to_string()).collect();

        write!(
            f,
            "<ColorSpace referenceSpaceType={:?}, name={}, ",
            impl_.reference_space_type, impl_.name
        )?;

        if impl_.aliases.len() == 1 {
            write!(f, "alias={}, ", impl_.aliases[0])?;
        } else if impl_.aliases.len() > 1 {
            write!(f, "aliases=[{}], ", impl_.aliases.join(", "))?;
        }

        if !impl_.family.is_empty() {
            write!(f, "family={}, ", impl_.family)?;
        }

        if !impl_.equality_group.is_empty() {
            write!(f, "equalityGroup={}, ", impl_.equality_group)?;
        }

        if impl_.bit_depth != BitDepth::Unknown {
            write!(f, "bitDepth={:?}, ", impl_.bit_depth)?;
        }

        write!(f, "isData={}", impl_.is_data)?;

        if num_vars > 0 {
            write!(
                f,
                ", allocation={:?}, vars={}",
                impl_.allocation,
                vars.join(" ")
            )?;
        }

        if impl_.categories.get_num_tokens() > 0 {
            let categories: Vec<String> = (0..impl_.categories.get_num_tokens())
                .map(|i| impl_.categories.get_token(i).to_string())
                .collect();
            write!(f, ", categories={}", categories.join(","))?;
        }

        if !impl_.encoding.is_empty() {
            write!(f, ", encoding={}", impl_.encoding)?;
        }

        if !impl_.description.is_empty() {
            write!(f, ", description={}", impl_.description)?;
        }

        if let Some(to_ref_transform) = &impl_.to_ref_transform {
            write!(f, ",\n    {} --> Reference\n        {}", impl_.name, to_ref_transform)?;
        }

        if let Some(from_ref_transform) = &impl_.from_ref_transform {
            write!(f, ",\n    Reference --> {}\n        {}", impl_.name, from_ref_transform)?;
        }

        write!(f, ">")
    }
}
