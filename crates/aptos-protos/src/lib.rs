// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use prost::Message;
use prost_types::{DescriptorProto, UninterpretedOption};
use move_core_types::{value::MoveTypeLayout, value::MoveStructLayout, u256};

mod pb;
pub use pb::aptos::*;
use move_core_types::account_address::AccountAddress;
use move_core_types::value::{MoveStruct, MoveValue};

fn make_url(option: &UninterpretedOption) -> String {
    let mut res = String::new();
    for name in option.name.iter() {
        res.push_str(&name.name_part);
    }
    res
}
/*
fn parse_layout(descriptor: &DescriptorProto) -> Option<MoveStructLayout> {
    let res = vec![];
    for field in descriptor.field.iter() {
        match field.label? {
            1 | 2 => {  // Optional/Required are singular values
                res.push(match field.r#type? {
                    4 => MoveTypeLayout::U64,  // uint64
                    8 => MoveTypeLayout::Bool,  // bool
                    12 => {  // bytes
                        let kind = descriptor.options?.uninterpreted_option.iter().find_map(|option| {
                            if (make_url(option) == "BcsOptions.Kind") {
                                return Some(option);
                            }
                            None
                        })?;
                        match kind.identifier_value?.as_str() {
                            "ADDRESS" => MoveTypeLayout::Address,
                            "SIGNER" => MoveTypeLayout::Signer,
                            "VEC_U8" => MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
                            "U128" => MoveTypeLayout::U128,
                            "U256" => MoveTypeLayout::U256,
                            default => return None,
                        }
                    },
                    10 | 11 => {  // message/group are structs
                        MoveTypeLayout::Struct(parse_layout(descriptor)?)
                    },
                    default => return None,
                });
            },
            3 => {  // repeated
                let elem_type = match field.r#type? {
                    4 => MoveTypeLayout::U64,
                    8 => MoveTypeLayout::Bool,
                    12 => {
                        let kind = descriptor.options?.uninterpreted_option.iter().find_map(|option| {
                            if (make_url(option) == "BcsOptions.Kind") {
                                return Some(option);
                            }
                            None
                        })?;
                        match kind.identifier_value?.as_str() {
                            "ADDRESS" => MoveTypeLayout::Address,
                            "SIGNER" => MoveTypeLayout::Signer,
                            "VEC_U8" => MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
                            "U128" => MoveTypeLayout::U128,
                            "U256" => MoveTypeLayout::U256,
                            default => return None,
                        }
                    },
                    10 | 11 => {

                    },
                    default => return None,
                };
                res.push(MoveTypeLayout::Vector(Box::new(elem_type)));
            }
            default=> return None,
        }
    }
    Some(MoveStructLayout::new(res))
}

pub fn move_layout(descriptor: &[u8]) -> Option<Vec<u8>> {
    let descriptor = DescriptorProto::decode(descriptor).unwrap();
    let mut res = vec![];
    serialize_layout(&descriptor, &mut res)?;
    Some(res)
}

pub fn reserialize(bcs: &[u8], layout: &[u8]) -> Vec<u8> {
    vec![]
}
*/
pub fn deserialize(bcs: &[u8], layout: &[u8]) -> Option<MoveStruct> {
    let layout = bcs::from_bytes::<MoveStructLayout>(layout).ok()?;
    MoveStruct::simple_deserialize(bcs, &layout).ok()
}

// Proto deserialization into MoveStruct given move struct layout

fn read_varint(proto: &mut &[u8]) -> Option<u64> {
    let mut res = 0;
    for i in 0 .. 10 {
        if i >= proto.len() {
            break;
        }
        let byte = proto[i] as u64;
        res |= (byte & 0x7f) << (i * 7);
        if byte & 0x80 == 0 {
            *proto = &proto[i + 1 ..];
            return Some(res)
        }
    };
    None
}

enum WireData<'a> {
    Varint(u64),
    LengthDelimited(&'a [u8]),
}

fn as_length_delimited<'a>(data: &WireData<'a>) -> Option<&'a [u8]> {
    match data {
        WireData::LengthDelimited(x) => Some(*x),
        _ => None,
    }
}

fn proto_deserialize_inner(proto: &mut &[u8], layout: &[MoveTypeLayout]) -> Option<MoveStruct> {
    let mut parsed_fields = vec![];
    while !proto.is_empty() {
        let tag = read_varint(proto)?;
        if tag == 0 || tag >= 1 << 29 {
            return None;
        }
        let field_idx = (tag >> 3) as u32 - 1;
        let data = match tag & 7 {
            0 => {  // varint
                let value = read_varint(proto)?;
                WireData::Varint(value)
            },
            2 => {  // length delimited
                let size = read_varint(proto)? as usize;
                let value = proto.get(..size)?;
                *proto = &proto[size..];
                WireData::LengthDelimited(value)
            },
            _ => return None,
        };
        parsed_fields.push((field_idx, data));
    }
    // Proto doesn't guarantee that fields are in field_num order, so we sort them.
    // However it's crucial to use a stable sort because the order for the same field number
    // corresponds with the ordering of repeated fields
    parsed_fields.sort_by_key(|(field_idx, _)| *field_idx);
    let as_varint = |data: Option<&WireData>| match data {
        Some(WireData::Varint(x)) => Some(*x),
        None => Some(0),
        _ => None,
    };
    let mut idx_in_parsed = 0;
    let mut res = vec![];
    let mut val_from_parsed = |i| {
        let field_idx = if idx_in_parsed < parsed_fields.len() { parsed_fields[idx_in_parsed].0 } else { u32::max_value() };
        // At this point i is always <= field_idx
        if i < field_idx as usize {
            None
        } else {
            // i must be equal to field_idx
            idx_in_parsed += 1;
            Some(&parsed_fields[idx_in_parsed - 1].1)
        }
    };
    for (i, field_layout) in layout.iter().enumerate() {
        use move_core_types::value::MoveTypeLayout::*;
        let val = val_from_parsed(i);
        match field_layout {
            Bool => res.push(MoveValue::Bool(as_varint(val)? != 0)),
            U8 => res.push(MoveValue::U8(as_varint(val)? as u8)),
            U16 => res.push(MoveValue::U16(as_varint(val)? as u16)),
            U32 => res.push(MoveValue::U32(as_varint(val)? as u32)),
            U64 => res.push(MoveValue::U64(as_varint(val)? as u64)),
            U128 => {
                let val = u128::from_le_bytes(as_length_delimited(val?)?.try_into().ok()?);
                res.push(MoveValue::U128(val));
            },
            U256 => {
                let val = u256::U256::from_le_bytes(as_length_delimited(val?)?.try_into().ok()?);
                res.push(MoveValue::U256(val));
            },
            Address => {
                let val = AccountAddress::from_bytes(as_length_delimited(val?)?).ok()?;
                res.push(MoveValue::Address(val));
            },
            Signer => {
                let val = AccountAddress::from_bytes(as_length_delimited(val?)?).ok()?;
                res.push(MoveValue::Signer(val));
            },
            Vector(elem_layout) => {
                let val = if let Some(mut val) = val {
                    match elem_layout.as_ref() {
                        Bool => {
                            let v = as_length_delimited(val)?;
                            v.iter().map(|x| MoveValue::Bool(*x != 0)).collect()
                        },
                        U8 => {
                            let v = as_length_delimited(val)?;
                            v.iter().map(|x| MoveValue::U8(*x)).collect()
                        },
                        _ => {
                            let mut vec = vec![];
                            loop {
                               let x = match elem_layout.as_ref() {
                                   U16 => MoveValue::U16(as_varint(Some(val))? as u16),
                                   U32 => MoveValue::U32(as_varint(Some(val))? as u32),
                                   U64 => MoveValue::U64(as_varint(Some(val))?),
                                   U128 => MoveValue::U128(u128::from_le_bytes(as_length_delimited(val)?.try_into().ok()?)),
                                   U256 => MoveValue::U256(u256::U256::from_le_bytes(as_length_delimited(val)?.try_into().ok()?)),
                                   Address => MoveValue::Address(AccountAddress::from_bytes(as_length_delimited(val)?).ok()?),
                                   Signer => MoveValue::Signer(AccountAddress::from_bytes(as_length_delimited(val)?).ok()?),
                                   Vector(inner_layout) => {
                                       let mut buffer = as_length_delimited(val)?;
                                       match inner_layout.as_ref() {
                                           Bool => {
                                               MoveValue::Vector(buffer.iter().map(|x| MoveValue::Bool(*x != 0)).collect())
                                           },
                                           U8 => {
                                               MoveValue::Vector(buffer.iter().map(|x| MoveValue::U8(*x)).collect())
                                           },
                                           _ => {
                                               let wrapper_layout = std::slice::from_ref(elem_layout.as_ref());
                                               let wrapper = proto_deserialize_inner(&mut buffer, wrapper_layout)?;
                                               wrapper.into_fields().pop()?
                                           },
                                       }
                                   },
                                   Struct(struct_layout) => {
                                       MoveValue::Struct(proto_deserialize(as_length_delimited(val)?, struct_layout)?)
                                   },
                                   _ => return None,  // cannot happen
                               };
                               vec.push(x);
                               val = match val_from_parsed(i) { Some(x) => x, None => break };
                            }
                            vec
                        }
                    }
                } else {
                    // no fields present, so empty vector
                    vec![]
                };
                res.push(MoveValue::Vector(val));
            },
            Struct(child_layout) => {
                let val = proto_deserialize(as_length_delimited(val?)?, child_layout)?;
                res.push(MoveValue::Struct(val));
            },
        }
    }
    Some(MoveStruct::new(res))
}

pub fn proto_deserialize(mut proto: &[u8], layout: &MoveStructLayout) -> Option<MoveStruct> {
    proto_deserialize_inner(&mut proto, layout.fields())
}

// Proto serialization of a MoveStruct
fn serialize_varint(mut x: u64, out: &mut Vec<u8>) {
    while x >= 128 {
        out.push((x | 128) as u8);
        x >>= 7;
    }
    out.push(x as u8);
}

fn encode_tag(field_num: u32, wire_type: u32, out: &mut Vec<u8>) {
    serialize_varint(((field_num << 3) | wire_type) as u64, out);
}

fn serialize_varint_field(field_num: u32, value: u64, out: &mut Vec<u8>) {
    encode_tag(field_num, 0, out);
    serialize_varint(value, out);
}

fn serialize_length_delim(field_num: u32, value: &[u8], out: &mut Vec<u8>) {
    encode_tag(field_num, 2, out);
    serialize_varint(value.len() as u64, out);
    out.extend_from_slice(value);
}

fn proto_serialize_vector_value(vec: &[MoveValue], field_num: u32, top_level: bool, out: &mut Vec<u8>) {
    if vec.is_empty() {
        // An empty vector at top level is either empty bytes fields which can be omitted
        // or has zero tag/value pairs which are thus not present. However, an empty vector not
        // at top-level, ie. a vector inside a vector still needs to be present. A vector inside
        // a vector is either a bool/u8 vector or a wrapped vector. Both representations are
        // given by a length-delimited empty string rep.
        if !top_level {
            serialize_length_delim(field_num, &[], out);
        }
        return;
    }
    let first = &vec[0];
    use MoveValue::*;
    match first {
        Bool(_) | U8(_) => {
            let mut bytes = vec![];
            for v in vec {
                bytes.push(match v {
                    Bool(v) => if *v { 1u8 } else { 0u8 },
                    U8(v) => *v,
                    _ => return,  // should never happen
                });
            }
            serialize_length_delim(field_num, &bytes, out);
        },
        Vector(_) if top_level => {
            // Difficult case we have a vector of vectors.
            for v in vec {
                let v = match v {
                    Vector(v) => v,
                    _ => return,  // should never happen
                };
                proto_serialize_vector_value(v, field_num, false, out);
            }
        }
        _  => {
            if top_level {
                for v in vec { proto_serialize_value(v, field_num, out); }
            } else {
                let mut wrapper = vec![];
                proto_serialize_vector_value(vec, 1, false, &mut wrapper);
                serialize_length_delim(field_num, &wrapper, out)
            }
        },
    }
}

fn proto_serialize_value(value: &MoveValue, field_num: u32, out: &mut Vec<u8>) {
    use MoveValue::*;
    match value {
        Bool(v) => serialize_varint_field(field_num, if *v { 1 } else { 0 }, out),
        U8(v) => serialize_varint_field(field_num, *v as u64, out),
        U16(v) => serialize_varint_field(field_num, *v as u64, out),
        U32(v) => serialize_varint_field(field_num, *v as u64, out),
        U64(v) => serialize_varint_field(field_num, *v as u64, out),
        U128(v) => serialize_length_delim(field_num, &v.to_le_bytes(), out),
        U256(v) => serialize_length_delim(field_num, &v.to_le_bytes(), out),
        Address(v) | Signer(v) => {
            serialize_length_delim(field_num, v.as_slice(), out);
        },
        Vector(v) => {
            proto_serialize_vector_value(v, field_num, true, out);
        },
        Struct(v) => {
            serialize_length_delim(field_num, &proto_serialize(v), out);
        },
    }
}

pub fn proto_serialize(value: &MoveStruct) -> Vec<u8> {
    use MoveStruct::*;
    let fields : Vec<&MoveValue> = match value {
        Runtime(fields) => fields.iter().map(|v| v).collect(),
        WithFields(fields) |
        WithTypes { fields, .. } => fields.iter().map(|(_, v)| v).collect(),
    };
    let mut res = vec![];
    for (i, field) in fields.into_iter().enumerate() {
        proto_serialize_value(field, i as u32 + 1, &mut res);
    };
    res
}
