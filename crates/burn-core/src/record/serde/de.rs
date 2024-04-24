use core::ptr;
use std::collections::HashMap;

use super::data::NestedValue;
use super::{adapter::BurnModuleAdapter, error::Error};

use serde::de::{EnumAccess, VariantAccess};
use serde::{
    de::{self, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor},
    forward_to_deserialize_any,
};

const RECORD_ITEM_SUFFIX: &str = "RecordItem";

/// A deserializer for the nested value data structure.
pub struct Deserializer<A: BurnModuleAdapter> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    value: Option<NestedValue>,
    default_for_missing_fields: bool,
    phantom: std::marker::PhantomData<A>,
}

impl<A: BurnModuleAdapter> Deserializer<A> {
    /// Creates a new deserializer with the given nested value.
    ///
    /// # Arguments
    ///
    /// * `value` - A nested value.
    /// * `default_for_missing_fields` - A boolean indicating whether to add missing fields with default value.
    pub fn new(value: NestedValue, default_for_missing_fields: bool) -> Self {
        Self {
            value: Some(value),
            default_for_missing_fields,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'de, A: BurnModuleAdapter> serde::Deserializer<'de> for Deserializer<A> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_any is not implemented")
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = match self.value {
            Some(value) => {
                // Adapt modules
                if let Some(name) = name.strip_suffix(RECORD_ITEM_SUFFIX) {
                    A::adapt(name, value)
                } else {
                    value
                }
            }
            None => {
                return Err(de::Error::custom(format!(
                    "Expected some value but got {:?}",
                    self.value
                )))
            }
        };

        match value {
            NestedValue::Map(map) => {
                // Add missing fields into the map with default value if needed.
                let map = if self.default_for_missing_fields {
                    let mut map = map;
                    for field in fields.iter().map(|s| s.to_string()) {
                        map.entry(field.clone())
                            .or_insert(NestedValue::Default(Some(field)));
                    }
                    map
                } else {
                    map
                };

                visitor.visit_map(HashMapAccess::<A>::new(
                    map,
                    self.default_for_missing_fields,
                ))
            }

            _ => Err(de::Error::custom(format!(
                "Expected struct but got {:?}",
                value
            ))),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.value.unwrap().as_string().unwrap().to_string())
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(NestedValue::Map(map)) => visitor.visit_map(HashMapAccess::<A>::new(
                map,
                self.default_for_missing_fields,
            )),

            _ => Err(de::Error::custom(format!(
                "Expected map value but got {:?}",
                self.value
            ))),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.value.unwrap().as_bool().unwrap())
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_i8 is not implemented")
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.value.unwrap().as_i16().unwrap().to_owned())
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.value.unwrap().as_i32().unwrap().to_owned())
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.value.unwrap().as_i64().unwrap().to_owned())
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_u8 is not implemented")
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.value.unwrap().as_u16().unwrap().to_owned())
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_u32 is not implemented")
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.value.unwrap().as_u64().unwrap().to_owned())
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.value.unwrap().as_f32().unwrap().to_owned())
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.value.unwrap().as_f64().unwrap().to_owned())
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_char is not implemented")
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.value.unwrap().as_string().unwrap().as_ref())
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_bytes is not implemented")
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_byte_buf is not implemented")
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(value) = self.value {
            visitor.visit_some(Deserializer::<A>::new(
                value,
                self.default_for_missing_fields,
            ))
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_unit is not implemented")
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_unit_struct is not implemented")
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(Deserializer::<A>::new(
            self.value.unwrap(),
            self.default_for_missing_fields,
        ))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(NestedValue::Vec(vec)) = self.value {
            visitor.visit_seq(VecSeqAccess::<A>::new(vec, self.default_for_missing_fields))
        } else {
            Err(de::Error::custom(format!(
                "Expected Vec but got {:?}",
                self.value
            )))
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_tuple is not implemented")
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_tuple_struct is not implemented")
    }

    /// Deserializes an enum by attempting to match its variants against the provided data.
    ///
    /// This function attempts to deserialize an enum by iterating over its possible variants
    /// and trying to deserialize the data into each until one succeeds. We need to do this
    /// because we don't have a way to know which variant to deserialize from the data.
    ///
    /// This is similar to Serde's
    /// [untagged enum deserialization](https://serde.rs/enum-representations.html#untagged),
    /// but it's on the deserializer side. Using `#[serde(untagged)]` on the enum will force
    /// using `deserialize_any`, which is not what we want because we want to use methods, such
    /// as `visit_struct`. Also we do not wish to use auto generate code for Deserialize just
    /// for enums because it will affect other serialization and deserialization, such
    /// as JSON and Bincode.
    ///
    /// # Safety
    /// The function uses an unsafe block to clone the `visitor`. This is necessary because
    /// the `Visitor` trait does not have a `Clone` implementation, and we need to clone it
    /// as we are going to use it multiple times. The Visitor is a code generated unit struct
    /// with no states or mutations, so it is safe to clone it in this case. We mainly care
    /// about the `visit_enum` method, which is the only method that will be called on the
    /// cloned visitor.
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        fn clone_unsafely<T>(thing: &T) -> T {
            unsafe {
                // Allocate memory for the clone.
                let clone = ptr::null_mut();
                // Correcting pointer usage based on feedback
                let clone = ptr::addr_of_mut!(*clone);
                // Copy the memory
                ptr::copy_nonoverlapping(thing as *const T, clone, 1);
                // Transmute the cloned data pointer into an owned instance of T.
                ptr::read(clone)
            }
        }

        // Try each variant in order
        for &variant in variants {
            // clone visitor to avoid moving it
            let cloned_visitor = clone_unsafely(&visitor);
            let result = cloned_visitor.visit_enum(ProbeEnumAccess::<A>::new(
                self.value.clone().unwrap(),
                variant.to_owned(),
                self.default_for_missing_fields,
            ));

            if result.is_ok() {
                return result;
            }
        }

        Err(de::Error::custom("No variant match"))
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_identifier is not implemented")
    }
}

/// A sequence access for a vector in the nested value data structure.
struct VecSeqAccess<A: BurnModuleAdapter> {
    iter: std::vec::IntoIter<NestedValue>,
    default_for_missing_fields: bool,
    phantom: std::marker::PhantomData<A>,
}

impl<A: BurnModuleAdapter> VecSeqAccess<A> {
    fn new(vec: Vec<NestedValue>, default_for_missing_fields: bool) -> Self {
        VecSeqAccess {
            iter: vec.into_iter(),
            default_for_missing_fields,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'de, A> SeqAccess<'de> for VecSeqAccess<A>
where
    NestedValueWrapper<A>: IntoDeserializer<'de, Error>,
    A: BurnModuleAdapter,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let item = match self.iter.next() {
            Some(v) => v,
            None => return Ok(None),
        };

        seed.deserialize(
            NestedValueWrapper::<A>::new(item, self.default_for_missing_fields).into_deserializer(),
        )
        .map(Some)
    }
}

/// A map access for a map in the nested value data structure.
struct HashMapAccess<A: BurnModuleAdapter> {
    iter: std::collections::hash_map::IntoIter<String, NestedValue>,
    next_value: Option<NestedValue>,
    default_for_missing_fields: bool,
    phantom: std::marker::PhantomData<A>,
}

impl<A: BurnModuleAdapter> HashMapAccess<A> {
    fn new(map: HashMap<String, NestedValue>, default_for_missing_fields: bool) -> Self {
        HashMapAccess {
            iter: map.into_iter(),
            next_value: None,
            default_for_missing_fields,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'de, A> MapAccess<'de> for HashMapAccess<A>
where
    String: IntoDeserializer<'de, Error>,
    NestedValueWrapper<A>: IntoDeserializer<'de, Error>,
    A: BurnModuleAdapter,
{
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((k, v)) => {
                // Keep the value for the next call to next_value_seed.
                self.next_value = Some(v);
                // Deserialize the key.
                seed.deserialize(k.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.next_value.take() {
            Some(NestedValue::Default(originator)) => {
                seed.deserialize(DefaultDeserializer::new(originator))
            }
            Some(v) => seed.deserialize(
                NestedValueWrapper::new(v, self.default_for_missing_fields).into_deserializer(),
            ),
            None => seed.deserialize(DefaultDeserializer::new(None)),
        }
    }
}

struct ProbeEnumAccess<A: BurnModuleAdapter> {
    value: NestedValue,
    current_variant: String,
    default_for_missing_fields: bool,
    phantom: std::marker::PhantomData<A>,
}

impl<A: BurnModuleAdapter> ProbeEnumAccess<A> {
    fn new(value: NestedValue, current_variant: String, default_for_missing_fields: bool) -> Self {
        ProbeEnumAccess {
            value,
            current_variant,
            default_for_missing_fields,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'de, A> EnumAccess<'de> for ProbeEnumAccess<A>
where
    A: BurnModuleAdapter,
{
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(self.current_variant.clone().into_deserializer())
            .map(|v| (v, self))
    }
}

impl<'de, A> VariantAccess<'de> for ProbeEnumAccess<A>
where
    A: BurnModuleAdapter,
{
    type Error = Error;

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(
            NestedValueWrapper::<A>::new(self.value, self.default_for_missing_fields)
                .into_deserializer(),
        )?;
        Ok(value)
    }

    fn unit_variant(self) -> Result<(), Self::Error> {
        unimplemented!("unit variant is not implemented because it is not used in the burn module")
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("tuple variant is not implemented because it is not used in the burn module")
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!(
            "struct variant is not implemented because it is not used in the burn module"
        )
    }
}

/// A wrapper for the nested value data structure with a burn module adapter.
struct NestedValueWrapper<A: BurnModuleAdapter> {
    value: NestedValue,
    default_for_missing_fields: bool,
    phantom: std::marker::PhantomData<A>,
}

impl<A: BurnModuleAdapter> NestedValueWrapper<A> {
    fn new(value: NestedValue, default_for_missing_fields: bool) -> Self {
        Self {
            value,
            default_for_missing_fields,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<A: BurnModuleAdapter> IntoDeserializer<'_, Error> for NestedValueWrapper<A> {
    type Deserializer = Deserializer<A>;

    fn into_deserializer(self) -> Self::Deserializer {
        Deserializer::<A>::new(self.value, self.default_for_missing_fields)
    }
}

/// A default deserializer that always returns the default value.
struct DefaultDeserializer {
    /// The originator field name (the top-level missing field name)
    originator_field_name: Option<String>,
}

impl DefaultDeserializer {
    fn new(originator_field_name: Option<String>) -> Self {
        Self {
            originator_field_name,
        }
    }
}

impl<'de> serde::Deserializer<'de> for DefaultDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(Default::default())
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(Default::default())
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(Default::default())
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(Default::default())
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(Default::default())
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(Default::default())
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(Default::default())
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(Default::default())
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_char(Default::default())
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(Default::default())
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(Default::default())
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(Default::default())
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(Default::default())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_none()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(DefaultSeqAccess::new(None))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(Default::default())
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Return an error if the originator field name is not set
        Err(Error::Other(
            format!(
                "Missing source values for the '{}' field of type '{}'. Please verify the source data and ensure the field name is correct",
                self.originator_field_name.unwrap_or("UNKNOWN".to_string()),
                 name,
            )
        ))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(DefaultSeqAccess::new(Some(len)))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(DefaultSeqAccess::new(Some(len)))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(DefaultMapAccess::new())
    }

    forward_to_deserialize_any! {
        u128 bytes byte_buf unit unit_struct newtype_struct
        enum identifier ignored_any
    }
}

/// A default sequence access that always returns None (empty sequence).
pub struct DefaultSeqAccess {
    size: Option<usize>,
}

impl Default for DefaultSeqAccess {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DefaultSeqAccess {
    /// Creates a new default sequence access with the given size hint.
    pub fn new(size: Option<usize>) -> Self {
        DefaultSeqAccess { size }
    }
}

impl<'de> SeqAccess<'de> for DefaultSeqAccess {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.size {
            Some(0) => Ok(None),
            Some(ref mut size) => {
                *size -= 1;
                seed.deserialize(DefaultDeserializer::new(None)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.size
    }
}

/// A default map access that always returns None (empty map).
pub struct DefaultMapAccess;

impl Default for DefaultMapAccess {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultMapAccess {
    /// Creates a new default map access.
    pub fn new() -> Self {
        DefaultMapAccess
    }
}

impl<'de> MapAccess<'de> for DefaultMapAccess {
    type Error = Error;

    fn next_key_seed<T>(&mut self, _seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        // Since this is a default implementation, we'll just return None.
        Ok(None)
    }

    fn next_value_seed<T>(&mut self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        unimplemented!("This should never be called since next_key_seed always returns None")
    }

    fn size_hint(&self) -> Option<usize> {
        // Since this is a default implementation, we'll just return None.
        None
    }
}
