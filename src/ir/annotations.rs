use clang;

#[derive(Copy, PartialEq, Clone, Debug)]
pub enum FieldAccessorKind {
    None,
    Regular,
    Unsafe,
    Immutable,
}

/// Annotations for a given item, or a field.
#[derive(Clone, PartialEq, Debug)]
pub struct Annotations {
    /// Whether this item is marked as opaque. Only applies to types.
    opaque: bool,
    /// Whether this item should be hidden from the output. Only applies to
    /// types.
    hide: bool,
    /// Whether this type should be replaced by another. The name must be the
    /// canonical name that that type would get.
    use_instead_of: Option<String>,
    /// Manually disable deriving copy/clone on this type. Only applies to
    /// struct or union types.
    disallow_copy: bool,
    /// Whether fields should be marked as private or not. You can set this on
    /// structs (it will apply to all the fields), or individual fields.
    private_fields: Option<bool>,
    /// The kind of accessor this field will have. Also can be applied to
    /// structs so all the fields inside share it by default.
    accessor_kind: Option<FieldAccessorKind>,
}

fn parse_accessor(s: &str) -> FieldAccessorKind {
    match s {
        "false" => FieldAccessorKind::None,
        "unsafe" => FieldAccessorKind::Unsafe,
        "immutable" => FieldAccessorKind::Immutable,
        _ => FieldAccessorKind::Regular,
    }
}

impl Default for Annotations {
    fn default() -> Self {
        Annotations {
            opaque: false,
            hide: false,
            use_instead_of: None,
            disallow_copy: false,
            private_fields: None,
            accessor_kind: None
        }
    }
}

impl Annotations {
    pub fn new(cursor: &clang::Cursor) -> Option<Annotations> {
        let mut anno = Annotations::default();
        let mut matched_one = false;
        anno.parse(&cursor.comment(), &mut matched_one);

        if matched_one {
            Some(anno)
        } else {
            None
        }
    }

    pub fn hide(&self) -> bool {
        self.hide
    }

    pub fn opaque(&self) -> bool {
        self.opaque
    }

    /// For a given type, indicates the type it should replace.
    ///
    /// For example, in the following code:
    ///
    /// ```cpp
    ///
    /// /** <div rustbindgen replaces="Bar"></div> */
    /// struct Foo { int x; };
    ///
    /// struct Bar { char foo; };
    /// ```
    ///
    /// the generated code would look something like:
    ///
    /// ```c++
    /// /** <div rustbindgen replaces="Bar"></div> */
    /// struct Bar {
    ///     int x;
    /// };
    /// ```
    ///
    /// That is, code for `Foo` is used to generate `Bar`.
    pub fn use_instead_of(&self) -> Option<&str> {
        self.use_instead_of.as_ref().map(|s| &**s)
    }

    pub fn disallow_copy(&self) -> bool {
        self.disallow_copy
    }

    pub fn private_fields(&self) -> Option<bool> {
        self.private_fields
    }

    pub fn accessor_kind(&self) -> Option<FieldAccessorKind> {
        self.accessor_kind
    }

    fn parse(&mut self, comment: &clang::Comment, matched: &mut bool) {
        use clangll::CXComment_HTMLStartTag;
        if comment.kind() == CXComment_HTMLStartTag &&
           comment.get_tag_name() == "div" &&
           comment.get_num_tag_attrs() > 1 &&
           comment.get_tag_attr_name(0) == "rustbindgen" {
            *matched = true;
            for i in 0..comment.get_num_tag_attrs() {
                let value = comment.get_tag_attr_value(i);
                let name = comment.get_tag_attr_name(i);
                match name.as_str() {
                    "opaque" => self.opaque = true,
                    "hide" => self.hide = true,
                    "nocopy" => self.disallow_copy = true,
                    "replaces" => self.use_instead_of = Some(value),
                    "private" => self.private_fields = Some(value != "false"),
                    "accessor"
                        => self.accessor_kind = Some(parse_accessor(&value)),
                    _ => {},
                }
            }
        }

        for i in 0..comment.num_children() {
            self.parse(&comment.get_child(i), matched);
        }
    }
}
