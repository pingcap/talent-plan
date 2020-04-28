/// A simple protobuf message.
pub struct Msg {
    #[prost(enumeration = "msg::Type", tag = "1")]
    pub r#type: i32,
    #[prost(uint64, tag = "2")]
    pub id: u64,
    #[prost(string, tag = "3")]
    pub name: std::string::String,
    #[prost(bytes, repeated, tag = "4")]
    pub paylad: ::std::vec::Vec<std::vec::Vec<u8>>,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::clone::Clone for Msg {
    #[inline]
    fn clone(&self) -> Msg {
        match *self {
            Msg {
                r#type: ref __self_0_0,
                id: ref __self_0_1,
                name: ref __self_0_2,
                paylad: ref __self_0_3,
            } => Msg {
                r#type: ::std::clone::Clone::clone(&(*__self_0_0)),
                id: ::std::clone::Clone::clone(&(*__self_0_1)),
                name: ::std::clone::Clone::clone(&(*__self_0_2)),
                paylad: ::std::clone::Clone::clone(&(*__self_0_3)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::PartialEq for Msg {
    #[inline]
    fn eq(&self, other: &Msg) -> bool {
        match *other {
            Msg {
                r#type: ref __self_1_0,
                id: ref __self_1_1,
                name: ref __self_1_2,
                paylad: ref __self_1_3,
            } => match *self {
                Msg {
                    r#type: ref __self_0_0,
                    id: ref __self_0_1,
                    name: ref __self_0_2,
                    paylad: ref __self_0_3,
                } => {
                    (*__self_0_0) == (*__self_1_0)
                        && (*__self_0_1) == (*__self_1_1)
                        && (*__self_0_2) == (*__self_1_2)
                        && (*__self_0_3) == (*__self_1_3)
                }
            },
        }
    }
    #[inline]
    fn ne(&self, other: &Msg) -> bool {
        match *other {
            Msg {
                r#type: ref __self_1_0,
                id: ref __self_1_1,
                name: ref __self_1_2,
                paylad: ref __self_1_3,
            } => match *self {
                Msg {
                    r#type: ref __self_0_0,
                    id: ref __self_0_1,
                    name: ref __self_0_2,
                    paylad: ref __self_0_3,
                } => {
                    (*__self_0_0) != (*__self_1_0)
                        || (*__self_0_1) != (*__self_1_1)
                        || (*__self_0_2) != (*__self_1_2)
                        || (*__self_0_3) != (*__self_1_3)
                }
            },
        }
    }
}
#[allow(non_snake_case, unused_attributes)]
mod Msg_MESSAGE {
    extern crate bytes as _bytes;
    extern crate prost as _prost;
    use super::*;
    impl _prost::Message for Msg {
        #[allow(unused_variables)]
        fn encode_raw<B>(&self, buf: &mut B)
        where
            B: _bytes::BufMut,
        {
            if self.r#type != super::msg::Type::default() as i32 {
                _prost::encoding::int32::encode(1u32, &self.r#type, buf);
            }
            if self.id != 0u64 {
                _prost::encoding::uint64::encode(2u32, &self.id, buf);
            }
            if self.name != "" {
                _prost::encoding::string::encode(3u32, &self.name, buf);
            }
            _prost::encoding::bytes::encode_repeated(4u32, &self.paylad, buf);
        }
        #[allow(unused_variables)]
        fn merge_field<B>(&mut self, buf: &mut B) -> ::std::result::Result<(), _prost::DecodeError>
        where
            B: _bytes::Buf,
        {
            const STRUCT_NAME: &'static str = "Msg";
            let (tag, wire_type) = _prost::encoding::decode_key(buf)?;
            match tag {
                1u32 => _prost::encoding::int32::merge(wire_type, &mut self.r#type, buf).map_err(
                    |mut error| {
                        error.push(STRUCT_NAME, "r#type");
                        error
                    },
                ),
                2u32 => _prost::encoding::uint64::merge(wire_type, &mut self.id, buf).map_err(
                    |mut error| {
                        error.push(STRUCT_NAME, "id");
                        error
                    },
                ),
                3u32 => _prost::encoding::string::merge(wire_type, &mut self.name, buf).map_err(
                    |mut error| {
                        error.push(STRUCT_NAME, "name");
                        error
                    },
                ),
                4u32 => _prost::encoding::bytes::merge_repeated(wire_type, &mut self.paylad, buf)
                    .map_err(|mut error| {
                        error.push(STRUCT_NAME, "paylad");
                        error
                    }),
                _ => _prost::encoding::skip_field(wire_type, buf),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0 + if self.r#type != super::msg::Type::default() as i32 {
                _prost::encoding::int32::encoded_len(1u32, &self.r#type)
            } else {
                0
            } + if self.id != 0u64 {
                _prost::encoding::uint64::encoded_len(2u32, &self.id)
            } else {
                0
            } + if self.name != "" {
                _prost::encoding::string::encoded_len(3u32, &self.name)
            } else {
                0
            } + _prost::encoding::bytes::encoded_len_repeated(4u32, &self.paylad)
        }
        fn clear(&mut self) {
            self.r#type = super::msg::Type::default() as i32;
            self.id = 0u64;
            self.name.clear();
            self.paylad.clear();
        }
    }
    impl Default for Msg {
        fn default() -> Msg {
            Msg {
                r#type: super::msg::Type::default() as i32,
                id: 0u64,
                name: ::std::string::String::new(),
                paylad: ::std::vec::Vec::new(),
            }
        }
    }
    impl ::std::fmt::Debug for Msg {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            let mut builder = f.debug_struct("Msg");
            let builder = {
                let wrapper = {
                    struct ScalarWrapper<'a>(&'a i32);
                    impl<'a> ::std::fmt::Debug for ScalarWrapper<'a> {
                        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                            match super::msg::Type::from_i32(*self.0) {
                                None => ::std::fmt::Debug::fmt(&self.0, f),
                                Some(en) => ::std::fmt::Debug::fmt(&en, f),
                            }
                        }
                    }
                    ScalarWrapper(&self.r#type)
                };
                builder.field("r#type", &wrapper)
            };
            let builder = {
                let wrapper = {
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.id)
                };
                builder.field("id", &wrapper)
            };
            let builder = {
                let wrapper = {
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.name)
                };
                builder.field("name", &wrapper)
            };
            let builder = {
                let wrapper = {
                    struct ScalarWrapper<'a>(&'a ::std::vec::Vec<::std::vec::Vec<u8>>);
                    impl<'a> ::std::fmt::Debug for ScalarWrapper<'a> {
                        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                            let mut vec_builder = f.debug_list();
                            for v in self.0 {
                                fn Inner<T>(v: T) -> T {
                                    v
                                }
                                vec_builder.entry(&Inner(v));
                            }
                            vec_builder.finish()
                        }
                    }
                    ScalarWrapper(&self.paylad)
                };
                builder.field("paylad", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(dead_code)]
    impl Msg {
        pub fn r#type(&self) -> super::msg::Type {
            super::msg::Type::from_i32(self.r#type).unwrap_or(super::msg::Type::default())
        }
        pub fn set_type(&mut self, value: super::msg::Type) {
            self.r#type = value as i32;
        }
    }
}
pub mod msg {
    #[repr(i32)]
    #[structural_match]
    #[rustc_copy_clone_marker]
    pub enum Type {
        Unknown = 0,
        Put = 1,
        Get = 2,
        Del = 3,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::clone::Clone for Type {
        #[inline]
        fn clone(&self) -> Type {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::marker::Copy for Type {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::fmt::Debug for Type {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match (&*self,) {
                (&Type::Unknown,) => {
                    let mut debug_trait_builder = f.debug_tuple("Unknown");
                    debug_trait_builder.finish()
                }
                (&Type::Put,) => {
                    let mut debug_trait_builder = f.debug_tuple("Put");
                    debug_trait_builder.finish()
                }
                (&Type::Get,) => {
                    let mut debug_trait_builder = f.debug_tuple("Get");
                    debug_trait_builder.finish()
                }
                (&Type::Del,) => {
                    let mut debug_trait_builder = f.debug_tuple("Del");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::cmp::PartialEq for Type {
        #[inline]
        fn eq(&self, other: &Type) -> bool {
            {
                let __self_vi = unsafe { ::std::intrinsics::discriminant_value(&*self) } as i32;
                let __arg_1_vi = unsafe { ::std::intrinsics::discriminant_value(&*other) } as i32;
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::cmp::Eq for Type {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::hash::Hash for Type {
        fn hash<__H: ::std::hash::Hasher>(&self, state: &mut __H) -> () {
            match (&*self,) {
                _ => ::std::hash::Hash::hash(
                    &unsafe { ::std::intrinsics::discriminant_value(self) },
                    state,
                ),
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::cmp::PartialOrd for Type {
        #[inline]
        fn partial_cmp(&self, other: &Type) -> ::std::option::Option<::std::cmp::Ordering> {
            {
                let __self_vi = unsafe { ::std::intrinsics::discriminant_value(&*self) } as i32;
                let __arg_1_vi = unsafe { ::std::intrinsics::discriminant_value(&*other) } as i32;
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => ::std::option::Option::Some(::std::cmp::Ordering::Equal),
                    }
                } else {
                    __self_vi.partial_cmp(&__arg_1_vi)
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::cmp::Ord for Type {
        #[inline]
        fn cmp(&self, other: &Type) -> ::std::cmp::Ordering {
            {
                let __self_vi = unsafe { ::std::intrinsics::discriminant_value(&*self) } as i32;
                let __arg_1_vi = unsafe { ::std::intrinsics::discriminant_value(&*other) } as i32;
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => ::std::cmp::Ordering::Equal,
                    }
                } else {
                    __self_vi.cmp(&__arg_1_vi)
                }
            }
        }
    }
    #[allow(non_snake_case, unused_attributes)]
    mod Type_ENUMERATION {
        use super::*;
        impl Type {
            ///Returns `true` if `value` is a variant of `Type`.
            pub fn is_valid(value: i32) -> bool {
                match value {
                    0 => true,
                    1 => true,
                    2 => true,
                    3 => true,
                    _ => false,
                }
            }
            ///Converts an `i32` to a `Type`, or `None` if `value` is not a valid variant.
            pub fn from_i32(value: i32) -> ::std::option::Option<Type> {
                match value {
                    0 => ::std::option::Option::Some(Type::Unknown),
                    1 => ::std::option::Option::Some(Type::Put),
                    2 => ::std::option::Option::Some(Type::Get),
                    3 => ::std::option::Option::Some(Type::Del),
                    _ => ::std::option::Option::None,
                }
            }
        }
        impl ::std::default::Default for Type {
            fn default() -> Type {
                Type::Unknown
            }
        }
        impl ::std::convert::From<Type> for i32 {
            fn from(value: Type) -> i32 {
                value as i32
            }
        }
    }
}
