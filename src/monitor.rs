macro_rules! dpi_vec2_impl {
    ($($t_ident:ident ($m1:ident, $m2:ident) $name:literal),* $(,)?) => {
        $(
            // Type definition
            document!(
                concat!("Represents an unscaled logical or physical ", $name, "."),
                #[derive(Copy, Clone, Debug)]
                pub enum $t_ident {
                    #[doc = "Logical"] #[doc = $name] #[doc = "that is scalable to monitor DPI."]
                    Logical(f64, f64),
                    #[doc = "Physical"] #[doc = $name] #[doc = "in absolute values regardless of DPI."]
                    Physical(u32, u32),
                }
            );

            // Implementations
            impl $t_ident {
                document!(
                    concat!(
                        "Gets `self` as a logical ", stringify!($m1), " and ", stringify!($m2),
                        ", downscaled with the given factor.\n\n",
                        "If `self` is already logical, no downscaling is done."
                    ),
                    #[inline]
                    pub fn as_logical(self, scale: Scale) -> (f64, f64) {
                        // NOTE: `const fn` doesn't have floating point arithmetic yet.
                        match self {
                            Self::Logical($m1, $m2) => ($m1, $m2),
                            Self::Physical($m1, $m2) => ($m1 as f64 / scale, $m2 as f64 / scale),
                        }
                    }
                );

                document!(
                    concat!(
                        "Gets `self` as a physical ", stringify!($m1), " and ", stringify!($m2),
                        ", upscaled with the given factor.\n\n",
                        "If `self` is already physical, no upscaling is done."
                    ),
                    #[inline]
                    pub fn as_physical(self, scale: Scale) -> (u32, u32) {
                        // NOTE: `const fn` doesn't have floating point arithmetic yet.
                        match self {
                            Self::Logical($m1, $m2) => (($m1 * scale) as u32, ($m2 * scale) as u32),
                            Self::Physical($m1, $m2) => ($m1, $m2),
                        }
                    }
                );

                document!(
                    concat!(
                        "Converts `self` to a logical ", $name,
                        ", downscaled with the given factor.\n\n",
                        "If `self` is already logical, no downscaling is done."
                    ),
                    #[inline]
                    pub fn to_logical(self, scale: Scale) -> Self {
                        // NOTE: `const fn` doesn't have floating point arithmetic yet.
                        let ($m1, $m2) = self.as_logical(scale);
                        Self::Logical($m1, $m2)
                    }
                );

                document!(
                    concat!(
                        "Converts `self` to a physical ", $name,
                        ", upscaled with the given factor.\n\n",
                        "If `self` is already physical, no upscaling is done."
                    ),
                    #[inline]
                    pub fn to_physical(self, scale: Scale) -> Self {
                        // NOTE: `const fn` doesn't have floating point arithmetic yet.
                        let ($m1, $m2) = self.as_physical(scale);
                        Self::Physical($m1, $m2)
                    }
                );

                // Private implementations
                pub(crate) fn scale_if_logical(self, scale: Scale) -> (f64, f64) {
                    match self {
                        Self::Logical($m1, $m2) => ($m1 * scale, $m2 * scale),
                        Self::Physical($m1, $m2) => ($m1 as f64, $m2 as f64),
                    }
                }
            }
        )*
    };
}

// This is where the magic happens.
dpi_vec2_impl! {
    Point(x, y) "point",
    Size(width, height) "size",
}

/// Represents a DPI scale factor to apply to a [`Point`] or [`Size`].
pub type Scale = f64;
