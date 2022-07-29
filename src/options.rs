#[derive(Clone)]
pub struct Options {
    allow_directory_listing: bool,
    //    directory_listing_default_index: Option<String>,
    prefix: String,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            allow_directory_listing: false,
            //            directory_listing_default_index: None,
            prefix: "/".to_string(),
        }
    }
}

impl Options {
    pub fn allow_directory_listing(&self) -> bool {
        self.allow_directory_listing
    }

    //    pub fn directory_listing_default_index(&self) -> Option<&String> {
    //        self.directory_listing_default_index.as_ref()
    //    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

#[derive(Clone)]
pub struct OptionsBuilder {
    options: Options,
}

impl OptionsBuilder {
    pub fn new() -> Self {
        OptionsBuilder {
            options: Options::default(),
        }
    }

    pub fn allow_directory_listing(mut self, allow: bool) -> Self {
        self.options.allow_directory_listing = allow;
        self
    }

    //    pub fn directory_listing_default_index(mut self, default_index: &str) -> Self {
    //        self.options.directory_listing_default_index = Some(default_index.to_string());
    //        self
    //    }

    pub fn prefix(mut self, prefix: &str) -> Self {
        self.options.prefix = prefix.to_string();
        self
    }
}

impl Into<Options> for OptionsBuilder {
    fn into(mut self) -> Options {
        if !self.options.prefix.ends_with('/') {
            self.options.prefix.push('/')
        }
        self.options
    }
}
