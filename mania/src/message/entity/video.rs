use super::prelude::*;
use crate::core::protos::service::oidb::{IndexNode, MsgInfo};

const DEFAULT_THUMB: [u8; 2643] = [
    0x1F, 0x8B, 0x08, 0x08, 0x0B, 0x68, 0xA6, 0x67, 0x02, 0xFF, 0x6F, 0x75, 0x74, 0x2E, 0x62, 0x69,
    0x6E, 0x00, 0x9D, 0x56, 0x5D, 0x70, 0x13, 0xD7, 0x15, 0x3E, 0x77, 0x77, 0xB5, 0x5E, 0xAF, 0x24,
    0xB3, 0xB6, 0x17, 0x23, 0x04, 0x01, 0xCB, 0x60, 0x77, 0xDD, 0xC4, 0x54, 0xC2, 0x0B, 0x38, 0x40,
    0xE9, 0x5A, 0xC2, 0x60, 0x3C, 0xF2, 0x54, 0xE0, 0x94, 0x06, 0x0D, 0x0F, 0xB6, 0x08, 0x8E, 0xE4,
    0xB6, 0x0C, 0x7F, 0x49, 0x48, 0xA6, 0x9D, 0xEC, 0x1A, 0xDB, 0x31, 0x6E, 0xF2, 0x40, 0x71, 0x5B,
    0xA1, 0xBE, 0x2C, 0x2E, 0x55, 0xD3, 0x4E, 0x5E, 0x28, 0xA6, 0x09, 0x7D, 0x72, 0xC0, 0x26, 0x4D,
    0x67, 0x3C, 0xA3, 0x60, 0x27, 0x05, 0x5A, 0x66, 0x80, 0xB6, 0xB4, 0xA5, 0x7D, 0x80, 0xA6, 0xFF,
    0x0F, 0x75, 0xCF, 0xB9, 0x2B, 0xD3, 0xCE, 0xF4, 0xA5, 0xAD, 0x76, 0x46, 0x24, 0x2B, 0xDF, 0xEF,
    0x9E, 0xF3, 0x9D, 0xEF, 0x7C, 0xE7, 0x2C, 0xDC, 0x58, 0xB8, 0x03, 0x5A, 0xD7, 0x8E, 0x5D, 0x3B,
    0x80, 0x31, 0x00, 0x86, 0x0F, 0x2C, 0xFC, 0x03, 0x96, 0xED, 0x3C, 0x78, 0xE8, 0xE0, 0xD1, 0xBE,
    0xE3, 0x07, 0x9F, 0xAB, 0xCF, 0xBC, 0x5C, 0xDF, 0x73, 0x28, 0x77, 0xB8, 0xEF, 0xD8, 0xF1, 0x83,
    0x0B, 0x3F, 0x83, 0x21, 0x50, 0x2B, 0x2A, 0x94, 0x0A, 0x59, 0x55, 0x14, 0xC5, 0xAF, 0xAA, 0xFE,
    0xAA, 0x90, 0x56, 0x15, 0x0C, 0x56, 0x3D, 0xB1, 0xB4, 0xAE, 0x3A, 0xB4, 0x66, 0x75, 0xE3, 0xDA,
    0x86, 0xD5, 0x0D, 0x91, 0xA6, 0x27, 0x37, 0x7D, 0xAA, 0xA9, 0xD9, 0x6C, 0x8E, 0x34, 0x44, 0xDB,
    0x63, 0xE6, 0xD3, 0x5B, 0xB6, 0x6D, 0xDB, 0xD6, 0xB8, 0x6E, 0xFB, 0xAE, 0xC4, 0xD6, 0xCE, 0x4D,
    0x9F, 0xDE, 0xB6, 0x85, 0xA9, 0x7E, 0x7F, 0x55, 0xB0, 0x6A, 0xA5, 0xA6, 0xAD, 0xDC, 0x62, 0x34,
    0x18, 0x5B, 0xFE, 0xE7, 0xCF, 0xC2, 0x14, 0x54, 0x2B, 0xEC, 0x55, 0x78, 0x28, 0xB2, 0x6A, 0x10,
    0xAA, 0x99, 0x58, 0xCD, 0x16, 0xA6, 0xD9, 0x04, 0x06, 0xEF, 0x63, 0xFC, 0x03, 0xE5, 0x0F, 0x13,
    0x44, 0xC9, 0x27, 0x57, 0x28, 0x95, 0xAA, 0x5F, 0x03, 0x81, 0x89, 0xA2, 0x20, 0x89, 0x3E, 0x9F,
    0x24, 0xE1, 0x2F, 0x5F, 0xC1, 0xDF, 0x40, 0xAA, 0xF6, 0xD5, 0x44, 0x62, 0xED, 0x72, 0xED, 0xEE,
    0xBE, 0x8A, 0x86, 0x23, 0xFA, 0x7A, 0xE7, 0xF4, 0x39, 0x65, 0x4D, 0xFC, 0xC2, 0xBB, 0x4B, 0xF7,
    0x7C, 0xF0, 0x70, 0x6D, 0x6B, 0xE6, 0xE8, 0x60, 0xA5, 0x5A, 0xB7, 0x2C, 0xB4, 0x3C, 0xDC, 0xD8,
    0xF4, 0x09, 0xA3, 0xF9, 0x93, 0xE6, 0x86, 0x8D, 0x9B, 0xDA, 0x9E, 0xDE, 0x9C, 0xD8, 0xDE, 0xB1,
    0x63, 0x67, 0xE7, 0xAE, 0xAE, 0x9E, 0x67, 0x3E, 0xB7, 0xF7, 0xF3, 0xCF, 0xEE, 0x4B, 0x1F, 0x78,
    0xEE, 0x60, 0xFF, 0xF3, 0xD9, 0xDC, 0xC0, 0xB1, 0xE3, 0x2F, 0xBC, 0xF8, 0xD2, 0x89, 0x97, 0x5F,
    0x39, 0x39, 0x34, 0x3C, 0xF2, 0xDA, 0xE8, 0xA9, 0xB1, 0xAF, 0x9D, 0x19, 0xFF, 0xFA, 0x37, 0xBE,
    0x99, 0x3F, 0x5B, 0x98, 0xF8, 0xF6, 0xF9, 0xEF, 0x14, 0xBF, 0xFB, 0xE6, 0xF7, 0xBE, 0xFF, 0x83,
    0x8B, 0x93, 0x97, 0x7E, 0xF8, 0xF6, 0x3B, 0x97, 0x7F, 0x74, 0xE5, 0xEA, 0xF4, 0xCC, 0xB5, 0xF7,
    0x7E, 0xFC, 0xFE, 0x4F, 0xAE, 0xCF, 0xCD, 0x7F, 0xF8, 0xD1, 0x4F, 0x6F, 0xDC, 0xBC, 0x75, 0xF7,
    0xDE, 0x2F, 0x7E, 0xF9, 0xAB, 0xFB, 0xBF, 0xFE, 0xCD, 0x6F, 0x1F, 0x3C, 0xFA, 0xC3, 0xC7, 0x7F,
    0xFC, 0xD3, 0x9F, 0xFF, 0xF2, 0xD7, 0xBF, 0xFD, 0x9D, 0x81, 0xC8, 0x16, 0x3F, 0xFF, 0x91, 0x0F,
    0x26, 0xCE, 0x04, 0x49, 0x12, 0xA5, 0x0A, 0xCA, 0x87, 0x09, 0x2F, 0xD1, 0x8F, 0xD5, 0x92, 0x2F,
    0x12, 0x93, 0x6B, 0xDA, 0x77, 0x57, 0xF4, 0x1D, 0xA9, 0x6D, 0x58, 0xEF, 0x28, 0x7A, 0xFC, 0xF4,
    0xB9, 0x0B, 0xEF, 0x56, 0xAE, 0x69, 0xDD, 0xF3, 0x70, 0x69, 0xE6, 0xE8, 0x07, 0x6A, 0xDD, 0x5A,
    0xF3, 0x6E, 0xE3, 0x23, 0x4A, 0x89, 0x67, 0xF4, 0xDF, 0x25, 0x34, 0xF8, 0x7F, 0x65, 0xF4, 0x38,
    0xA1, 0xC7, 0xF9, 0x2C, 0xDC, 0x82, 0x80, 0xC8, 0xB0, 0x60, 0x62, 0x35, 0x7C, 0x06, 0xEE, 0xB1,
    0xB7, 0xDE, 0x78, 0x4F, 0x76, 0x2B, 0x44, 0xCB, 0x5F, 0x80, 0xD6, 0x64, 0x7B, 0x01, 0x92, 0x25,
    0x71, 0x97, 0xAB, 0xDC, 0x28, 0x49, 0x4F, 0x65, 0x97, 0xD7, 0x86, 0xE1, 0xF5, 0x52, 0xAC, 0x19,
    0x52, 0x42, 0x89, 0x25, 0x7D, 0xAE, 0x60, 0xAB, 0x52, 0x63, 0xB4, 0xC6, 0xA9, 0xB4, 0x2A, 0x93,
    0x82, 0x53, 0xA5, 0xC3, 0x5D, 0x6B, 0xF1, 0xEB, 0x91, 0x75, 0x8A, 0xBD, 0x69, 0xEA, 0x61, 0x38,
    0x3C, 0x00, 0xA9, 0xAC, 0x76, 0xB9, 0xC8, 0x0A, 0x10, 0x9F, 0x87, 0x2B, 0xF3, 0x6C, 0xC5, 0x06,
    0xD8, 0x7E, 0xCD, 0x0E, 0x75, 0x8B, 0xA1, 0x2C, 0xE3, 0x48, 0xF5, 0xC5, 0x58, 0x56, 0x4B, 0xB1,
    0xA4, 0xE4, 0x4A, 0xA6, 0xD2, 0x05, 0x5D, 0xBD, 0x3A, 0x3C, 0x90, 0x0C, 0xB7, 0x42, 0x85, 0xC3,
    0x6E, 0xC5, 0x18, 0x9C, 0x70, 0x13, 0x17, 0xF7, 0x15, 0x46, 0x72, 0xF6, 0x65, 0xD7, 0x77, 0xDB,
    0x1A, 0xBE, 0x6D, 0xC9, 0x0F, 0x7A, 0xCD, 0x4B, 0x10, 0xBD, 0x04, 0xB1, 0x37, 0x20, 0x5A, 0x80,
    0x3E, 0x3C, 0x06, 0xE3, 0x8A, 0x01, 0x2E, 0x05, 0x54, 0x67, 0xD7, 0xE4, 0x31, 0x1C, 0x6B, 0xC8,
    0x15, 0xF2, 0x7E, 0x4B, 0x35, 0xB4, 0xBB, 0xF8, 0xC5, 0xEE, 0xD9, 0xAB, 0x0C, 0xED, 0x91, 0xE5,
    0xEB, 0x1F, 0x68, 0xCA, 0xD9, 0xFD, 0xF6, 0x32, 0x13, 0x0E, 0xB9, 0xD2, 0x6D, 0x4B, 0x49, 0xE7,
    0x83, 0x4D, 0x18, 0x8A, 0x10, 0xCE, 0x32, 0x83, 0x05, 0x85, 0x1A, 0xBB, 0x06, 0x01, 0x54, 0x48,
    0x43, 0xB3, 0x9D, 0x12, 0xF0, 0xA9, 0x5F, 0x44, 0x99, 0xB0, 0x57, 0xF9, 0xA0, 0x4D, 0x95, 0xB6,
    0xAA, 0x18, 0x8F, 0x7C, 0xFF, 0xE2, 0x86, 0x5A, 0x33, 0x90, 0x9C, 0x83, 0x59, 0x68, 0xCA, 0xE2,
    0x57, 0xC4, 0x54, 0x72, 0x26, 0x8C, 0xE4, 0x83, 0x6E, 0x1C, 0x0F, 0x45, 0x30, 0x14, 0x0A, 0xA4,
    0x2E, 0xEF, 0x83, 0x46, 0x7C, 0x74, 0x85, 0xD0, 0x4A, 0x91, 0xA9, 0x94, 0x68, 0xBB, 0x15, 0x8E,
    0x55, 0x55, 0xA6, 0x2A, 0x30, 0xDB, 0x3F, 0xB0, 0xC9, 0xBC, 0xD6, 0x08, 0xCB, 0x5D, 0x69, 0x96,
    0xBE, 0xF0, 0x89, 0x87, 0x7B, 0x1B, 0x29, 0x0C, 0xA4, 0xD4, 0x07, 0x2D, 0xA0, 0xC3, 0xE4, 0x54,
    0x17, 0x20, 0x5E, 0xBB, 0x25, 0x37, 0xDB, 0x19, 0x47, 0xA5, 0x20, 0xE0, 0x84, 0x0E, 0x0F, 0x0D,
    0xED, 0x8E, 0xC1, 0x3A, 0xA0, 0x2D, 0x25, 0x35, 0x5D, 0xDC, 0x88, 0x07, 0xF8, 0x5D, 0x3A, 0x8C,
    0x43, 0x2A, 0x12, 0x8C, 0xAE, 0xA0, 0xF4, 0xCB, 0x10, 0x2D, 0x51, 0x15, 0x01, 0x66, 0x20, 0xE4,
    0xB6, 0x8F, 0x41, 0xCC, 0xF2, 0x8F, 0x65, 0x74, 0x24, 0x56, 0x57, 0x10, 0x04, 0x11, 0xE6, 0x61,
    0x12, 0x26, 0x59, 0xB0, 0xFF, 0x8B, 0x18, 0x68, 0xB4, 0x06, 0x53, 0xE7, 0xB7, 0xC5, 0x7B, 0x58,
    0x91, 0x8E, 0x0B, 0x2A, 0xEC, 0x4F, 0x31, 0xCB, 0x17, 0x5D, 0xA6, 0xC3, 0x3B, 0xAE, 0xA8, 0x43,
    0xC6, 0x91, 0x11, 0x4B, 0x50, 0x7B, 0x0D, 0x37, 0x31, 0x83, 0x97, 0x8B, 0xC3, 0x74, 0x03, 0x3E,
    0x89, 0x7D, 0x85, 0x53, 0xD6, 0xB4, 0x2B, 0x15, 0x99, 0x45, 0xE7, 0x28, 0x20, 0x25, 0xAB, 0x8D,
    0xC3, 0xA4, 0x62, 0xB0, 0x23, 0x18, 0x87, 0x3E, 0x95, 0x6A, 0x86, 0xDF, 0xF7, 0xF0, 0x38, 0xE2,
    0x08, 0x39, 0x98, 0xE7, 0xA5, 0x59, 0x42, 0x3C, 0xF8, 0xFE, 0x85, 0x40, 0x42, 0xEA, 0xB3, 0xBC,
    0x9B, 0x51, 0x09, 0x99, 0x1A, 0x7B, 0x99, 0x8E, 0x08, 0x59, 0xAC, 0xED, 0x0C, 0x18, 0x18, 0x44,
    0x11, 0x79, 0xC4, 0x4A, 0x01, 0x92, 0x44, 0x08, 0x8F, 0x3C, 0xBD, 0xF9, 0xF9, 0xA3, 0x7D, 0xD4,
    0x3F, 0x91, 0x62, 0x1C, 0x43, 0xB0, 0xBD, 0x5C, 0x62, 0x49, 0x9E, 0xF4, 0x30, 0x29, 0x15, 0x52,
    0x08, 0xC4, 0x72, 0x36, 0x12, 0x04, 0x66, 0x09, 0x6B, 0x5B, 0x8C, 0x10, 0x7F, 0x22, 0xE5, 0x21,
    0x94, 0x50, 0xD0, 0x56, 0xF5, 0xDE, 0xFE, 0x73, 0x74, 0x39, 0x34, 0xF1, 0xF2, 0x7B, 0xA9, 0x04,
    0x7A, 0x34, 0xA2, 0x00, 0x23, 0x34, 0xB8, 0xC0, 0xD4, 0x5E, 0xB3, 0x14, 0x29, 0x62, 0x7E, 0x83,
    0x5E, 0x15, 0x5F, 0x1B, 0x93, 0xF9, 0xF9, 0xEB, 0x50, 0x94, 0x5E, 0x38, 0xEB, 0x62, 0x01, 0x98,
    0x75, 0x15, 0x0B, 0x81, 0x75, 0x2B, 0x57, 0x6E, 0x10, 0x6B, 0xA3, 0x13, 0xCF, 0xF8, 0x8C, 0x93,
    0x4A, 0x9A, 0x1C, 0x54, 0x5D, 0xAD, 0x39, 0x2D, 0x38, 0x4B, 0xB0, 0x0E, 0xA4, 0xF1, 0xB4, 0x4C,
    0xB5, 0x4E, 0x89, 0x4F, 0xB6, 0x5A, 0x3C, 0xD4, 0x18, 0xE6, 0x90, 0x79, 0xAC, 0xC0, 0x41, 0x7C,
    0x63, 0x84, 0xE8, 0x38, 0x15, 0x57, 0x43, 0xC9, 0x73, 0xBD, 0x9D, 0x2D, 0x46, 0xDA, 0xA9, 0x0C,
    0x9D, 0x93, 0x04, 0x42, 0x41, 0x8A, 0x35, 0xF6, 0x8A, 0x7D, 0x05, 0xEF, 0xB6, 0x1E, 0x46, 0x69,
    0xC6, 0x88, 0x2A, 0xEA, 0x09, 0xC4, 0x90, 0xE9, 0x75, 0x8A, 0xFA, 0x30, 0x37, 0x8A, 0xC7, 0xAF,
    0x64, 0xB5, 0xEB, 0x59, 0xAD, 0xDF, 0x86, 0xDC, 0xE8, 0xE9, 0x6E, 0xE8, 0x2C, 0xDA, 0xAB, 0x82,
    0xB0, 0x99, 0x5A, 0x63, 0x1C, 0x86, 0x4A, 0xC2, 0xDE, 0x7E, 0x07, 0x29, 0xC5, 0xB4, 0xA3, 0xDA,
    0x18, 0xE4, 0x5C, 0xB9, 0x44, 0x7C, 0x20, 0xC8, 0x01, 0x52, 0x93, 0x2B, 0x9A, 0xD0, 0x6F, 0x87,
    0xBB, 0x63, 0x98, 0x47, 0xB8, 0x5B, 0xC0, 0x06, 0x0A, 0x94, 0x84, 0x93, 0xD6, 0x12, 0xD9, 0x5E,
    0x55, 0xB4, 0xD7, 0xCA, 0x4E, 0x95, 0x9C, 0xF7, 0x93, 0x71, 0x08, 0xFD, 0x76, 0xDD, 0x45, 0x13,
    0x4F, 0xB0, 0xAC, 0xF6, 0xA0, 0x37, 0xE2, 0xCA, 0xD4, 0xAD, 0x19, 0xCC, 0x89, 0x74, 0x72, 0x85,
    0xB2, 0x7A, 0xDC, 0x3C, 0xD7, 0x73, 0x36, 0x46, 0xF3, 0x73, 0xAA, 0xEE, 0x15, 0x7B, 0x65, 0x10,
    0xB6, 0x86, 0xE1, 0x84, 0x09, 0x77, 0x72, 0xA3, 0xFB, 0x4B, 0x8C, 0x7B, 0xD0, 0xEF, 0x38, 0x35,
    0x75, 0x4E, 0x55, 0x11, 0xB2, 0xAC, 0x68, 0x87, 0x72, 0xF6, 0x55, 0xAB, 0x12, 0x2D, 0xCA, 0xBA,
    0x4A, 0xB1, 0x04, 0xBC, 0xBF, 0x12, 0x6A, 0xAD, 0x91, 0x33, 0x96, 0xD2, 0x1D, 0x0B, 0x70, 0x4B,
    0xE0, 0x0D, 0x22, 0x3A, 0xE4, 0x66, 0x55, 0xB3, 0x31, 0x32, 0x34, 0xEB, 0xA4, 0xC9, 0x5A, 0xFA,
    0x07, 0xB8, 0xA8, 0xD8, 0xFC, 0x94, 0xC1, 0x8A, 0xF9, 0x6A, 0xF4, 0x92, 0x92, 0x78, 0xCC, 0x92,
    0xA9, 0x93, 0x97, 0x7B, 0xA9, 0x91, 0x11, 0x68, 0x25, 0xB1, 0x3B, 0xC6, 0x45, 0x3C, 0x3E, 0x95,
    0xB6, 0xDA, 0xC2, 0x68, 0x63, 0x6C, 0x6A, 0x8E, 0xDD, 0x74, 0xE3, 0xF7, 0xF1, 0xBA, 0xFA, 0x59,
    0xF8, 0xEC, 0xBE, 0xC2, 0xB0, 0x40, 0x66, 0x91, 0xEE, 0x6D, 0x11, 0xC2, 0x39, 0x67, 0x45, 0x6A,
    0x7D, 0xAB, 0xC5, 0x2B, 0x0B, 0x67, 0x5D, 0x5F, 0x19, 0x8B, 0x8A, 0x18, 0x86, 0x44, 0x32, 0xD1,
    0x2D, 0x50, 0x8F, 0x62, 0x99, 0xED, 0xC3, 0x26, 0xB8, 0x1E, 0x8E, 0x43, 0x2F, 0x9E, 0x7D, 0x6B,
    0x56, 0xE0, 0x9E, 0xB3, 0x1B, 0x3D, 0x54, 0xE1, 0x30, 0x1B, 0x5D, 0x39, 0x4C, 0xFA, 0xC7, 0x78,
    0x31, 0x68, 0x6E, 0x27, 0xDA, 0x9C, 0x36, 0x87, 0xA2, 0x68, 0xA7, 0xCE, 0xC6, 0xBF, 0xDE, 0x4F,
    0x7C, 0x57, 0xC9, 0xC8, 0x07, 0x29, 0xBF, 0xEE, 0x0B, 0x65, 0xDD, 0xA4, 0x7B, 0x75, 0x25, 0x61,
    0x05, 0xCC, 0x6B, 0x98, 0x91, 0x42, 0x45, 0xEB, 0x01, 0xFE, 0xBE, 0xB5, 0x53, 0xC8, 0x0F, 0x71,
    0x9F, 0x9C, 0xD6, 0xC9, 0x29, 0x25, 0x6E, 0x50, 0x2B, 0xA9, 0x05, 0xA9, 0xC8, 0x69, 0x2F, 0x04,
    0x03, 0xB0, 0x2C, 0x98, 0xF1, 0x5C, 0xF2, 0x40, 0x37, 0x44, 0xC7, 0x20, 0x6E, 0x30, 0x6C, 0x38,
    0xDE, 0x09, 0x5E, 0x43, 0x1A, 0xAC, 0x79, 0xD4, 0xA0, 0x76, 0x18, 0xE2, 0xE7, 0x75, 0xAF, 0xFF,
    0x75, 0xE5, 0x6D, 0x64, 0x82, 0xF4, 0x4B, 0xD7, 0x91, 0x8B, 0xC3, 0x5C, 0x12, 0xBF, 0xF2, 0xAE,
    0x34, 0x13, 0x0D, 0x77, 0x80, 0x39, 0x06, 0x5D, 0x12, 0x99, 0x13, 0x75, 0x54, 0x32, 0x5E, 0xC2,
    0xF2, 0x63, 0x57, 0x50, 0x9F, 0x71, 0x93, 0x4F, 0x4B, 0x3A, 0x3B, 0x74, 0x71, 0x03, 0xD7, 0x95,
    0x50, 0xB6, 0xCC, 0x8D, 0xFC, 0x5F, 0x6C, 0xC3, 0xA1, 0x94, 0xD8, 0xE9, 0xCA, 0xA7, 0xAD, 0x91,
    0x12, 0xE3, 0x2E, 0xC2, 0x6F, 0x1F, 0xA4, 0xFA, 0x38, 0x8B, 0x2D, 0xD9, 0x6E, 0xD5, 0x7D, 0xBC,
    0xEF, 0x5B, 0x7E, 0x12, 0x67, 0x2F, 0x19, 0xE0, 0x08, 0x9F, 0x00, 0x3E, 0x1E, 0x4F, 0x75, 0x12,
    0xCF, 0x06, 0x1B, 0xEC, 0x70, 0x87, 0x50, 0x8B, 0xEE, 0x65, 0x07, 0x52, 0x6C, 0x6C, 0xD1, 0x14,
    0x86, 0x66, 0xEA, 0xF1, 0x1D, 0xDA, 0x00, 0x23, 0x0E, 0x84, 0x96, 0xFB, 0xEE, 0x0D, 0xA2, 0x5B,
    0x6A, 0xA9, 0x57, 0x51, 0xAA, 0x18, 0xFF, 0x2D, 0x9C, 0x02, 0xE3, 0xBC, 0x6B, 0x7B, 0xD8, 0xF9,
    0x72, 0x1C, 0x95, 0x63, 0x52, 0x88, 0x1B, 0x21, 0x87, 0xC2, 0xFF, 0x96, 0x66, 0xA0, 0xC9, 0xD0,
    0x78, 0xAA, 0xAA, 0x84, 0x5C, 0xB0, 0xE7, 0x37, 0xC8, 0x13, 0x27, 0x2D, 0x35, 0x08, 0x2D, 0x9C,
    0xB2, 0x04, 0xA6, 0x4C, 0x6C, 0x33, 0x72, 0xAA, 0xC9, 0x29, 0xA4, 0xF4, 0x3C, 0x86, 0x85, 0x38,
    0x18, 0x81, 0x4C, 0x07, 0x11, 0x2C, 0x4F, 0xEE, 0x86, 0xBE, 0x98, 0xB1, 0x6B, 0xE9, 0x7F, 0xE2,
    0x99, 0x3C, 0xFE, 0x6C, 0x08, 0x5F, 0x9E, 0x0F, 0xCB, 0x28, 0xB9, 0x92, 0x70, 0x1F, 0x8B, 0x5C,
    0x89, 0x85, 0x61, 0x14, 0x0F, 0x7A, 0x0D, 0xB1, 0x8E, 0xAE, 0x9E, 0xFC, 0x37, 0x20, 0x46, 0xF9,
    0xF0, 0x94, 0xEF, 0x79, 0x49, 0xE6, 0xD2, 0x58, 0x3E, 0x2D, 0x5F, 0x14, 0x32, 0xF6, 0xEA, 0xC2,
    0x49, 0x6B, 0x16, 0xDA, 0xC2, 0x12, 0x6A, 0x78, 0x7F, 0x29, 0xD2, 0x5A, 0x76, 0x48, 0x15, 0x76,
    0x5B, 0x6A, 0xB3, 0xFD, 0x00, 0x95, 0x57, 0xB6, 0x2A, 0x72, 0xCD, 0x30, 0x09, 0x7E, 0x82, 0xA6,
    0x97, 0x33, 0xC2, 0x51, 0x1B, 0x6C, 0x34, 0x00, 0xBC, 0x6A, 0xCF, 0x53, 0x06, 0x43, 0xBB, 0x71,
    0xCE, 0x0B, 0x24, 0xDB, 0xC6, 0x28, 0x25, 0xD7, 0x6C, 0x4F, 0x7A, 0xD3, 0x7E, 0x12, 0x16, 0xFD,
    0x12, 0xCB, 0x16, 0x71, 0x13, 0x59, 0xEC, 0xAE, 0x0E, 0xAC, 0x4E, 0x06, 0xCB, 0x26, 0x97, 0x53,
    0x1E, 0xD9, 0x63, 0x74, 0xC9, 0x38, 0x37, 0x50, 0x29, 0x24, 0x41, 0xC1, 0xA1, 0x7A, 0xA6, 0x65,
    0x4F, 0x03, 0x50, 0x16, 0x25, 0xB6, 0x5D, 0x01, 0xD5, 0xD2, 0x44, 0x8E, 0x2F, 0xDB, 0xB8, 0x3E,
    0xBC, 0x98, 0x0D, 0x71, 0xFB, 0xDD, 0x68, 0xAF, 0x39, 0x1F, 0xA3, 0xA6, 0xC5, 0x47, 0xA3, 0xA6,
    0x2A, 0x31, 0x6E, 0xAC, 0x6D, 0xA9, 0x58, 0x07, 0x9F, 0x1F, 0xDE, 0xF4, 0x3B, 0xEB, 0x26, 0x76,
    0xD1, 0x34, 0xE5, 0x63, 0x9E, 0xB6, 0x0F, 0xD3, 0xDE, 0x4F, 0x04, 0x8E, 0x50, 0x73, 0x37, 0xC2,
    0xCE, 0xCE, 0x3E, 0x3E, 0x2A, 0xC8, 0x51, 0xBB, 0x24, 0x6E, 0x93, 0x78, 0x91, 0x1E, 0xD8, 0xCD,
    0xDF, 0x96, 0x83, 0xEF, 0x46, 0x4B, 0x42, 0xF7, 0x4E, 0x92, 0xA5, 0xD0, 0x61, 0xF4, 0x44, 0x9B,
    0x37, 0x22, 0x7F, 0xD3, 0xD5, 0x51, 0x49, 0x1B, 0xC5, 0x3C, 0x3A, 0x88, 0x81, 0x54, 0xF5, 0x60,
    0x40, 0x7C, 0xF5, 0xC8, 0xD0, 0x08, 0x21, 0x87, 0xF1, 0x71, 0x8C, 0xA8, 0x56, 0xF2, 0x3A, 0x67,
    0x78, 0x56, 0x0C, 0x10, 0x48, 0xC0, 0xFB, 0x55, 0xCE, 0xC1, 0xE4, 0x18, 0x51, 0xEF, 0xC7, 0xA9,
    0x44, 0xFB, 0x4E, 0xAF, 0xEE, 0x71, 0xE7, 0x83, 0x75, 0xD9, 0xD0, 0x65, 0x6E, 0x3F, 0x8E, 0x4C,
    0xEB, 0x48, 0xB3, 0x3D, 0x62, 0xAF, 0xE0, 0x23, 0xA8, 0x0E, 0x0F, 0x47, 0x67, 0xF1, 0x77, 0xD6,
    0xEA, 0xB9, 0xD4, 0xAE, 0x3D, 0xA9, 0x94, 0x80, 0x53, 0x03, 0x05, 0x17, 0x29, 0xE2, 0x30, 0x22,
    0x87, 0x35, 0x78, 0x18, 0xB3, 0xA2, 0x8A, 0x69, 0x71, 0x3F, 0xE4, 0xFB, 0x14, 0x52, 0x82, 0x92,
    0xC1, 0x21, 0xE2, 0x0C, 0x97, 0x1B, 0xD0, 0xAA, 0x7D, 0x66, 0x9C, 0x12, 0x1E, 0x76, 0xE3, 0x73,
    0xB8, 0xF2, 0x78, 0x61, 0x46, 0x8C, 0xD0, 0x38, 0x1F, 0x0D, 0xFB, 0xCB, 0x39, 0x44, 0xCB, 0x90,
    0x3C, 0xC7, 0xF2, 0xFC, 0xE3, 0xDB, 0x85, 0xF4, 0x61, 0x93, 0x83, 0x21, 0xA1, 0x75, 0xE0, 0x66,
    0x82, 0x5D, 0x4D, 0xDB, 0x0A, 0x39, 0xA5, 0x88, 0xAE, 0x47, 0xD1, 0x4F, 0x2D, 0x22, 0xF0, 0x3F,
    0xA7, 0x75, 0xC4, 0xF0, 0x2C, 0x28, 0xED, 0x01, 0x0A, 0xAF, 0x9C, 0x69, 0x47, 0x5F, 0xE6, 0x00,
    0xD3, 0xDE, 0x71, 0x0E, 0x10, 0xC4, 0x92, 0xBD, 0xCE, 0x59, 0x6E, 0xE5, 0x95, 0xE0, 0xC1, 0x0E,
    0xB9, 0xDE, 0x20, 0xE7, 0x1B, 0x40, 0x1C, 0x23, 0x8D, 0x19, 0xC2, 0x00, 0x9D, 0xCF, 0x6A, 0x13,
    0x8E, 0x1F, 0xCD, 0x90, 0x16, 0x2E, 0x28, 0x1B, 0x96, 0x49, 0x1C, 0x2D, 0x27, 0xE3, 0xE5, 0x7B,
    0x93, 0x92, 0xE6, 0x86, 0xC5, 0x69, 0x11, 0xB8, 0x4F, 0x12, 0xE8, 0x9C, 0xF5, 0xFE, 0x40, 0x37,
    0xAD, 0xAD, 0x46, 0x31, 0x82, 0x3A, 0x8A, 0x90, 0xA3, 0x25, 0xAC, 0x20, 0x3A, 0x96, 0x75, 0x0D,
    0x6B, 0x2B, 0xAF, 0x23, 0x7A, 0x5B, 0x50, 0x55, 0x91, 0x2C, 0xBF, 0x13, 0x6D, 0x58, 0x26, 0x43,
    0x6A, 0xC1, 0x22, 0xD4, 0xD9, 0x5E, 0x43, 0x23, 0x02, 0xCD, 0xAF, 0xD7, 0xB0, 0xDF, 0x05, 0x5C,
    0xD4, 0x92, 0xB4, 0x69, 0x21, 0xC2, 0x55, 0x6B, 0x1A, 0x3D, 0x03, 0x87, 0xEC, 0x65, 0x92, 0xEE,
    0xCA, 0xA0, 0xA0, 0x73, 0x10, 0x1C, 0xC3, 0x38, 0xF8, 0x4A, 0x51, 0x0C, 0x62, 0x99, 0xA9, 0x70,
    0x76, 0x70, 0x81, 0x11, 0xD7, 0x9E, 0xCF, 0x12, 0xAA, 0x9E, 0xC2, 0xE9, 0x8D, 0x6D, 0xAD, 0x18,
    0x7C, 0x5C, 0x8B, 0x41, 0x31, 0x9C, 0x0D, 0xDD, 0x22, 0x9D, 0x64, 0xB1, 0xDB, 0x70, 0x6E, 0xE1,
    0x2D, 0x9B, 0xF1, 0xDE, 0x04, 0xB5, 0x7F, 0x50, 0x24, 0x32, 0xB8, 0x49, 0x2D, 0x77, 0xFB, 0x10,
    0x47, 0xE8, 0xDA, 0xE1, 0x58, 0x83, 0x0E, 0x2E, 0x42, 0x08, 0x32, 0xEA, 0xA5, 0x5B, 0xC4, 0x16,
    0x0E, 0xF1, 0x58, 0xC6, 0x03, 0xAD, 0xD6, 0x30, 0x5F, 0x51, 0x68, 0x93, 0x26, 0xFA, 0x84, 0xB0,
    0x6C, 0x96, 0x3C, 0xB1, 0x71, 0xA2, 0x90, 0x64, 0x69, 0xF2, 0xAB, 0x35, 0x44, 0x87, 0xA9, 0xA4,
    0x33, 0x24, 0x9D, 0x71, 0x08, 0xD3, 0xB8, 0x2F, 0xC8, 0x51, 0xDC, 0x1D, 0xC4, 0x15, 0xDC, 0xA4,
    0xB1, 0x65, 0x91, 0x90, 0x2B, 0xB4, 0x5B, 0x63, 0xA9, 0x10, 0x84, 0x9B, 0x39, 0xF5, 0x60, 0x3D,
    0xCA, 0xBE, 0x21, 0xD1, 0x79, 0xC1, 0x40, 0x65, 0x0E, 0xCF, 0x0A, 0x4F, 0x14, 0x3C, 0x0C, 0xC5,
    0x08, 0xCD, 0x61, 0xC0, 0x21, 0x4E, 0x6A, 0x92, 0xEC, 0x28, 0xC5, 0xC7, 0x3A, 0xAF, 0xA3, 0xC0,
    0x69, 0xEC, 0x2A, 0x57, 0x27, 0x0B, 0xB4, 0xED, 0xB7, 0x5D, 0x37, 0x72, 0x34, 0x77, 0xF5, 0x70,
    0x46, 0x0F, 0x23, 0xAD, 0x81, 0x12, 0xC9, 0x46, 0x32, 0x03, 0x39, 0x5C, 0x17, 0x62, 0xB4, 0x70,
    0x50, 0xAF, 0x4E, 0xE0, 0xC4, 0x11, 0x4A, 0x31, 0x8E, 0xC1, 0xED, 0x1C, 0x9D, 0xE9, 0x4B, 0x02,
    0xAD, 0xEC, 0x7A, 0x0F, 0xDA, 0xCD, 0xAB, 0x0B, 0x37, 0xFF, 0x09, 0xD9, 0x2F, 0x90, 0x2B, 0x36,
    0x0F, 0x00, 0x00,
];

#[derive(Default)]
pub struct VideoEntity {
    pub file_name: String,
    pub video_hash: String,
    pub height: i32,
    pub width: i32,
    pub video_size: i32,
    pub video_length: i32,
    pub video_url: String,
    // TODO: stream (video, thumb)
    pub(crate) node: Option<IndexNode>, // for download, 2025/02/08
    pub(crate) video_uuid: Option<String>,
    pub(crate) msg_info: Option<MsgInfo>,
    pub(crate) compat: Option<VideoFile>,
}

impl Debug for VideoEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "[Video]: {}x{}: {} {}",
            self.width, self.height, self.video_size, self.video_url
        )
    }
}

impl Display for VideoEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[视频]")
    }
}

impl MessageEntity for VideoEntity {
    fn pack_element(&self) -> Vec<Elem> {
        todo!()
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        elem.video_file.as_ref().map(|video_file| {
            dda!(Self {
                video_hash: hex::encode(&video_file.file_md5),
                height: video_file.file_height,
                width: video_file.file_width,
                video_size: video_file.file_size,
                video_uuid: Some(video_file.file_uuid.to_owned()),
            })
        });
        match elem.common_elem.as_ref() {
            Some(common) => {
                match (
                    common.service_type,
                    common.pb_elem.as_ref(),
                    common.business_type,
                ) {
                    (48, pb, _) => {
                        let msg_info = MsgInfo::decode(pb).ok()?;
                        let msg_info_body = msg_info.msg_info_body.first();
                        let node = msg_info_body?.index.to_owned()?;
                        let info = node.info.as_ref()?;
                        Some(dda!(Self {
                            file_name: info.file_name.clone(),
                            height: info.height as i32,
                            width: info.width as i32,
                            video_size: info.file_size as i32,
                            video_length: info.time as i32,
                            node: Some(node),
                        }))
                    }
                    _ => None,
                }
            }
            None => None,
        }
    }
}
