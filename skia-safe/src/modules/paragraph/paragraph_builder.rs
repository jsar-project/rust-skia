use std::{fmt, os::raw, ptr, str};

use skia_bindings as sb;

use super::{FontCollection, Paragraph, ParagraphStyle, PlaceholderStyle, TextStyle};
use crate::prelude::*;

pub type ParagraphBuilder = RefHandle<sb::skia_textlayout_ParagraphBuilder>;
unsafe_send_sync!(ParagraphBuilder);

pub type UnicodePosition = sb::SkUnicode_Position;
pub type LineBreakType = sb::SkUnicode_LineBreakType;
variant_name!(LineBreakType::SoftLineBreak);
variant_name!(LineBreakType::HardLineBreak);

impl NativeDrop for sb::skia_textlayout_ParagraphBuilder {
    fn drop(&mut self) {
        unsafe { sb::C_ParagraphBuilder_delete(self) }
    }
}

impl fmt::Debug for ParagraphBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParagraphBuilder").finish()
    }
}

impl ParagraphBuilder {
    pub fn push_style(&mut self, style: &TextStyle) -> &mut Self {
        unsafe { sb::C_ParagraphBuilder_pushStyle(self.native_mut(), style.native()) }
        self
    }

    pub fn pop(&mut self) -> &mut Self {
        unsafe { sb::C_ParagraphBuilder_pop(self.native_mut()) }
        self
    }

    pub fn peek_style(&mut self) -> TextStyle {
        let mut ts = TextStyle::default();
        unsafe { sb::C_ParagraphBuilder_peekStyle(self.native_mut(), ts.native_mut()) }
        ts
    }

    pub fn add_text(&mut self, str: impl AsRef<str>) -> &mut Self {
        let str = str.as_ref();
        unsafe {
            sb::C_ParagraphBuilder_addText(
                self.native_mut(),
                str.as_ptr() as *const raw::c_char,
                str.len(),
            )
        }
        self
    }

    pub fn add_placeholder(&mut self, placeholder_style: &PlaceholderStyle) -> &mut Self {
        unsafe {
            sb::C_ParagraphBuilder_addPlaceholder(self.native_mut(), placeholder_style.native())
        }
        self
    }

    pub fn build(&mut self) -> Paragraph {
        Paragraph::from_ptr(unsafe { sb::C_ParagraphBuilder_Build(self.native_mut()) }).unwrap()
    }

    pub fn get_text(&mut self) -> &str {
        unsafe {
            let mut ptr = ptr::null_mut();
            let mut len = 0;
            sb::C_ParagraphBuilder_getText(self.native_mut(), &mut ptr, &mut len);
            // ptr may indeed be `null` if there is no text.
            str::from_utf8_unchecked(safer::from_raw_parts(ptr as *const u8, len))
        }
    }

    pub fn get_paragraph_style(&self) -> ParagraphStyle {
        ParagraphStyle::from_ptr(unsafe { sb::C_ParagraphBuilder_getParagraphStyle(self.native()) })
            .unwrap()
    }

    #[cfg(feature = "textlayout-client-icu")]
    pub fn set_words_utf8(&mut self, words: &[UnicodePosition]) -> &mut Self {
        unsafe {
            sb::C_ParagraphBuilder_setWordsUtf8(self.native_mut(), words.as_ptr(), words.len())
        }
        self
    }

    #[cfg(feature = "textlayout-client-icu")]
    pub fn set_grapheme_breaks_utf8(&mut self, graphemes: &[UnicodePosition]) -> &mut Self {
        unsafe {
            sb::C_ParagraphBuilder_setGraphemeBreaksUtf8(
                self.native_mut(),
                graphemes.as_ptr(),
                graphemes.len(),
            )
        }
        self
    }

    #[cfg(feature = "textlayout-client-icu")]
    pub fn set_line_breaks_utf8(
        &mut self,
        line_breaks: &[(UnicodePosition, LineBreakType)],
    ) -> &mut Self {
        let mut positions = Vec::with_capacity(line_breaks.len());
        let mut types = Vec::with_capacity(line_breaks.len());
        for &(position, break_type) in line_breaks {
            positions.push(position);
            types.push(break_type);
        }
        unsafe {
            sb::C_ParagraphBuilder_setLineBreaksUtf8(
                self.native_mut(),
                positions.as_ptr(),
                types.as_ptr(),
                line_breaks.len(),
            )
        }
        self
    }

    // TODO: Wrap setWordsUtf16, setGraphemeBreaksUtf16, setLineBreaksUtf16,
    // getClientICUData, setUnicode.

    pub fn reset(&mut self) {
        unsafe { sb::C_ParagraphBuilder_Reset(self.native_mut()) }
    }

    pub fn new(style: &ParagraphStyle, font_collection: impl Into<FontCollection>) -> Self {
        #[cfg(all(feature = "embed-icudtl", not(feature = "textlayout-client-icu")))]
        crate::icu::init();

        Self::from_ptr(unsafe {
            sb::C_ParagraphBuilder_make(style.native(), font_collection.into().into_ptr())
        })
        .expect("Unicode initialization error")
    }
}
