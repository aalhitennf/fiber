#![allow(clippy::missing_errors_doc, clippy::many_single_char_names)]

use std::str::FromStr;

use fiber_macro::StyleParser;
use floem::cosmic_text::Weight;
use floem::peniko::Color;
use floem::prop;
use floem::style::{
    AlignContentProp, AlignItemsProp, AlignSelf, AspectRatio, Background, BorderBottom, BorderColor, BorderLeft,
    BorderRadius, BorderRight, BorderTop, BoxShadow, BoxShadowProp, Cursor, CursorColor, CursorStyle, DisplayProp,
    FlexBasis, FlexDirectionProp, FlexGrow, FlexShrink, FlexWrapProp, FontFamily, FontSize, FontStyle, FontWeight, Gap,
    Height, InsetBottom, InsetLeft, InsetRight, InsetTop, JustifyContentProp, JustifySelf, LineHeight, MarginBottom,
    MarginLeft, MarginRight, MarginTop, MaxHeight, MaxWidth, MinHeight, MinWidth, Outline, OutlineColor, PaddingBottom,
    PaddingLeft, PaddingRight, PaddingTop, PositionProp, Style, TextColor, TextOverflow, TextOverflowProp, Transition,
    Width, ZIndex,
};
use floem::taffy::{AlignContent, AlignItems, Display, FlexDirection, FlexWrap, JustifyContent, Position, Size};
use floem::unit::{Pct, Px, PxPct, PxPctAuto};
use floem::views::scroll::Border;
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;

lazy_static! {
    // Matches Css comment blocks /* */
    static ref COMMENT_REGEX: Regex = Regex::new(r"\/\*[^\*]+\*\/").unwrap();
    // Matches everything inside brackets (..)
    static ref BRACKETS_REGEX: Regex = Regex::new(r"\(([^)]+)\)").unwrap();
    // Matches everything inside braces {..}
    static ref BRACES_REGEX: Regex = Regex::new(r"\{([^}]+)\}").unwrap();
}

#[derive(Debug, Clone)]
pub struct StyleError {
    pub error: String,
    pub value: String,
}

impl StyleError {
    pub fn new(error: &(impl ToString + ?Sized), value: &(impl ToString + ?Sized)) -> Self {
        Self {
            error: error.to_string(),
            value: value.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Selector {
    Active,
    Focus,
    Hover,
    Disabled,
}

impl FromStr for Selector {
    type Err = StyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Selector::Active),
            "focus" => Ok(Selector::Focus),
            "hover" => Ok(Selector::Hover),
            "disabled" => Ok(Selector::Disabled),
            _ => Err(StyleError::new("unsupported selector", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClassSelector {
    pub class: String,
    pub selector: Option<Selector>,
}

impl FromStr for ClassSelector {
    type Err = StyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(StyleError::new("empty class selector", s));
        }

        if !s.contains(':') {
            return Ok(ClassSelector {
                class: s.to_string(),
                selector: None,
            });
        }

        let Some((class, selector)) = s.split_once(':') else {
            return Err(StyleError::new("invalid class selector", s));
        };

        let class = class.trim();
        let selector = selector.trim();

        if class.is_empty() || selector.is_empty() {
            return Err(StyleError::new("invalid class selector", s));
        }

        let selector = Selector::from_str(selector)?;

        Ok(ClassSelector {
            class: class.to_string(),
            selector: Some(selector),
        })
    }
}

#[derive(Debug, Clone)]
pub struct StyleBlock {
    pub selectors: Vec<ClassSelector>,
    pub props: Vec<StyleProperty>,
    pub errors: Vec<StyleError>,
}

impl From<StyleBlock> for Style {
    fn from(value: StyleBlock) -> Self {
        value
            .props
            .into_iter()
            .fold(Style::new(), |s, p| match StyleProps::try_from(p) {
                Ok(v) => v.apply_style(s),
                Err(e) => {
                    eprintln!("{e:?}");
                    s
                }
            })
    }
}

#[derive(Debug, Clone)]
pub struct StyleProperty {
    pub key: String,
    pub value: String,
}

impl FromStr for StyleProperty {
    type Err = StyleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, val) = s
            .trim()
            .split_once(':')
            .ok_or_else(|| StyleError::new("Expecting key value pair with :", s))?;
        Ok(Self {
            key: key.trim().to_string(),
            value: val.trim().to_string(),
        })
    }
}

impl FromStr for StyleBlock {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (class, rest) = s.split_once('{').ok_or("Missing opening token {")?;

        if class.is_empty() {
            return Err("Class must contain value".into());
        }

        let mut selectors = class.split(',').flat_map(ClassSelector::from_str).collect::<Vec<_>>();

        selectors.sort();

        let (props, errors) = rest
            .split_inclusive(';')
            .filter_map(|s| {
                let st = s.trim().replace([';', '{', '}'], "");
                (!st.is_empty()).then_some(st)
            })
            .map(|s| StyleProperty::from_str(&s))
            .fold((Vec::new(), Vec::new()), |(mut props, mut errors), res| {
                match res {
                    Ok(prop) => props.push(prop),
                    Err(e) => errors.push(e),
                };
                (props, errors)
            });

        Ok(StyleBlock {
            selectors,
            props,
            errors,
        })
    }
}

/// Very naive css parser
pub struct StyleParser;

impl StyleParser {
    pub fn blocks(buf: &str) -> Vec<StyleBlock> {
        let mut buf = COMMENT_REGEX.replace_all(buf, "").to_string();

        if let Some(Some(root_block)) = buf
            .clone()
            .split_inclusive('}')
            .find(|b| b.contains(":root"))
            .map(|s| BRACES_REGEX.captures(s))
        {
            match root_block.get(1) {
                Some(root) => {
                    let vars = root
                        .as_str()
                        .split_inclusive(';')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .filter_map(|s| {
                            let split = s.split_once(':');
                            if split.is_none() {
                                warn!("Invalid :root variable {s}");
                            }
                            split
                        })
                        .map(|(k, v)| (k.trim(), v.trim()));

                    for (k, v) in vars {
                        let replaced = buf.replace(&format!("var({k})"), v);
                        let _ = std::mem::replace(&mut buf, replaced);
                    }
                }

                None => {
                    log::warn!("Invalid root block");
                }
            }
        } else {
            log::warn!("Invalid root block");
        }

        let blocks = buf
            .split_inclusive('}')
            .filter(|s| !s.contains(":root"))
            .map(StyleBlock::from_str)
            .filter_map(|res| res.inspect_err(|e| log::warn!("{e}")).ok())
            .collect::<Vec<_>>();

        blocks
    }
}

prop!(pub Padding: PxPctAuto {} = PxPctAuto::Px(0.0));
prop!(pub Margin: PxPctAuto {} = PxPctAuto::Px(0.0));
prop!(pub TransitionProp: f64 {} = 0.0);

#[derive(StyleParser)]
pub enum StyleProps {
    #[key("display")]
    #[parser("parse_display")]
    #[prop(DisplayProp)]
    Display(Display),

    #[key("position")]
    #[parser("parse_position")]
    #[prop(PositionProp)]
    Position(Position),

    #[key("width")]
    #[parser("parse_pxpctauto")]
    #[prop(Width)]
    Width(PxPctAuto),

    #[key("height")]
    #[parser("parse_pxpctauto")]
    #[prop(Height)]
    Height(PxPctAuto),

    #[key("min-width")]
    #[parser("parse_pxpctauto")]
    #[prop(MinWidth)]
    MinWidth(PxPctAuto),

    #[key("min-height")]
    #[parser("parse_pxpctauto")]
    #[prop(MinHeight)]
    MinHeight(PxPctAuto),

    #[key("max-width")]
    #[parser("parse_pxpctauto")]
    #[prop(MaxWidth)]
    MaxWidth(PxPctAuto),

    #[key("max-height")]
    #[parser("parse_pxpctauto")]
    #[prop(MaxHeight)]
    MaxHeight(PxPctAuto),

    #[key("flex-direction")]
    #[parser("parse_flex_direction")]
    #[prop(FlexDirectionProp)]
    FlexDirection(FlexDirection),

    #[key("flex-wrap")]
    #[parser("parse_flex_wrap")]
    #[prop(FlexWrapProp)]
    FlexWrap(FlexWrap),

    #[key("flex-grow")]
    #[parser("parse_f32")]
    #[prop(FlexGrow)]
    FlexGrow(f32),

    #[key("flex-shrink")]
    #[parser("parse_f32")]
    #[prop(FlexShrink)]
    FlexShrink(f32),

    #[key("flex-basis")]
    #[parser("parse_pxpctauto")]
    #[prop(FlexBasis)]
    FlexBasis(PxPctAuto),

    #[key("justify-content")]
    #[parser("parse_justify_content")]
    #[prop(JustifyContentProp)]
    JustifyContent(JustifyContent),

    #[key("justify-self")]
    #[parser("parse_align_items")]
    #[prop(JustifySelf)]
    JustifySelf(AlignItems),

    #[key("align-items")]
    #[parser("parse_align_items")]
    #[prop(AlignItemsProp)]
    AlignItems(AlignItems),

    #[key("align-content")]
    #[parser("parse_align_content")]
    #[prop(AlignContentProp)]
    AlignContent(AlignContent),

    #[key("align-self")]
    #[parser("parse_align_items")]
    #[prop(AlignSelf)]
    AlignSelf(AlignItems),

    #[key("border")]
    #[parser("parse_px")]
    #[prop(Border)]
    Border(Px),

    #[key("border-left")]
    #[parser("parse_px")]
    #[prop(BorderLeft)]
    BorderLeft(Px),

    #[key("border-top")]
    #[parser("parse_px")]
    #[prop(BorderTop)]
    BorderTop(Px),

    #[key("border-right")]
    #[parser("parse_px")]
    #[prop(BorderRight)]
    BorderRight(Px),

    #[key("border-bottom")]
    #[parser("parse_px")]
    #[prop(BorderBottom)]
    BorderBottom(Px),

    #[key("border-radius")]
    #[parser("parse_px_pct")]
    #[prop(BorderRadius)]
    BorderRadius(PxPct),

    #[key("outline-color")]
    #[parser("parse_color")]
    #[prop(OutlineColor)]
    OutlineColor(Color),

    #[key("outline")]
    #[parser("parse_px")]
    #[prop(Outline)]
    Outline(Px),

    #[key("border-color")]
    #[parser("parse_color")]
    #[prop(BorderColor)]
    BorderColor(Color),

    #[key("padding")]
    #[parser("parse_px_pct")]
    #[prop(Padding)]
    Padding(PxPct),

    #[key("padding-left")]
    #[parser("parse_px_pct")]
    #[prop(PaddingLeft)]
    PaddingLeft(PxPct),

    #[key("padding-top")]
    #[parser("parse_px_pct")]
    #[prop(PaddingTop)]
    PaddingTop(PxPct),

    #[key("padding-right")]
    #[parser("parse_px_pct")]
    #[prop(PaddingRight)]
    PaddingRight(PxPct),

    #[key("padding-bottom")]
    #[parser("parse_px_pct")]
    #[prop(PaddingBottom)]
    PaddingBottom(PxPct),

    #[key("margin")]
    #[parser("parse_pxpctauto")]
    #[prop(Margin)]
    Margin(PxPctAuto),

    #[key("margin-left")]
    #[parser("parse_pxpctauto")]
    #[prop(MarginLeft)]
    MarginLeft(PxPctAuto),

    #[key("margin-top")]
    #[parser("parse_pxpctauto")]
    #[prop(MarginTop)]
    MarginTop(PxPctAuto),

    #[key("margin-right")]
    #[parser("parse_pxpctauto")]
    #[prop(MarginRight)]
    MarginRight(PxPctAuto),

    #[key("margin-bottom")]
    #[parser("parse_pxpctauto")]
    #[prop(MarginBottom)]
    MarginBottom(PxPctAuto),

    #[key("left")]
    #[parser("parse_pxpctauto")]
    #[prop(InsetLeft)]
    InsetLeft(PxPctAuto),

    #[key("top")]
    #[parser("parse_pxpctauto")]
    #[prop(InsetTop)]
    InsetTop(PxPctAuto),

    #[key("right")]
    #[parser("parse_pxpctauto")]
    #[prop(InsetRight)]
    InsetRight(PxPctAuto),

    #[key("bottom")]
    #[parser("parse_pxpctauto")]
    #[prop(InsetBottom)]
    InsetBottom(PxPctAuto),

    #[key("z-index")]
    #[parser("parse_i32")]
    #[prop(ZIndex)]
    ZIndex(i32),

    #[key("cursor")]
    #[parser("parse_cursor_style")]
    #[prop(Cursor)]
    Cursor(CursorStyle),

    #[key("color")]
    #[parser("parse_color")]
    #[prop(TextColor)]
    Color(Color),

    #[key("background-color")]
    #[parser("parse_color")]
    #[prop(Background)]
    BackgroundColor(Color),

    #[key("box-shadow")]
    #[parser("parse_box_shadow")]
    #[prop(BoxShadowProp)]
    BoxShadow(BoxShadow),

    #[key("font-size")]
    #[parser("parse_f32")]
    #[prop(FontSize)]
    FontSize(f32),

    #[key("font-family")]
    #[parser("to_owned")]
    #[prop(FontFamily)]
    FontFamily(String),

    #[key("font-weight")]
    #[parser("parse_font_weight")]
    #[prop(FontWeight)]
    FontWeight(Weight),

    #[key("font-style")]
    #[parser("parse_font_style")]
    #[prop(FontStyle)]
    FontStyle(floem::cosmic_text::Style),

    #[key("cursor-color")]
    #[parser("parse_color")]
    #[prop(CursorColor)]
    CursorColor(Color),

    #[key("text-wrap")]
    #[parser("parse_text_overflow")]
    #[prop(TextOverflowProp)]
    TextOverflow(TextOverflow),

    #[key("line-height")]
    #[parser("parse_f32")]
    #[prop(LineHeight)]
    LineHeight(f32),

    #[key("aspect-ratio")]
    #[parser("parse_f32")]
    #[prop(AspectRatio)]
    AspectRatio(f32),

    #[key("gap")]
    #[parser("parse_gap")]
    #[prop(Gap)]
    Gap(Size<Px>),

    #[key("transition")]
    #[parser("parse_transition")]
    #[prop(TransitionProp)]
    Transition((String, Transition)),
}

impl StyleProps {
    pub fn apply_style(self, s: Style) -> Style {
        match self {
            StyleProps::Display(d) => s.display(d),
            StyleProps::Position(p) => s.position(p),
            StyleProps::Width(v) => s.width(v),
            StyleProps::Height(v) => s.height(v),
            StyleProps::MinWidth(v) => s.min_width(v),
            StyleProps::MinHeight(v) => s.min_height(v),
            StyleProps::MaxWidth(v) => s.max_width(v),
            StyleProps::MaxHeight(v) => s.max_height(v),
            StyleProps::FlexDirection(f) => s.flex_direction(f),
            StyleProps::FlexWrap(f) => s.flex_wrap(f),
            StyleProps::FlexGrow(f) => s.flex_grow(f),
            StyleProps::FlexShrink(f) => s.flex_shrink(f),
            StyleProps::FlexBasis(v) => s.flex_basis(v),
            StyleProps::JustifyContent(j) => s.justify_content(j),
            StyleProps::JustifySelf(a) => s.justify_self(a),
            StyleProps::AlignItems(a) => s.align_items(a),
            StyleProps::AlignContent(v) => s.align_content(v),
            StyleProps::AlignSelf(v) => s.align_self(v),
            StyleProps::Border(v) => s.border(v),
            StyleProps::BorderLeft(v) => s.border_left(v),
            StyleProps::BorderTop(v) => s.border_top(v),
            StyleProps::BorderRight(v) => s.border_right(v),
            StyleProps::BorderBottom(v) => s.border_bottom(v),
            StyleProps::BorderRadius(v) => s.border_radius(v),
            StyleProps::OutlineColor(v) => s.outline_color(v),
            StyleProps::Outline(v) => s.outline(v),
            StyleProps::BorderColor(v) => s.border_color(v),
            StyleProps::Padding(v) => s.padding(v),
            StyleProps::PaddingLeft(v) => s.padding_left(v),
            StyleProps::PaddingTop(v) => s.padding_top(v),
            StyleProps::PaddingRight(v) => s.padding_right(v),
            StyleProps::PaddingBottom(v) => s.padding_bottom(v),
            StyleProps::Margin(v) => s.margin(v),
            StyleProps::MarginLeft(v) => s.margin_left(v),
            StyleProps::MarginTop(v) => s.margin_top(v),
            StyleProps::MarginRight(v) => s.margin_right(v),
            StyleProps::MarginBottom(v) => s.margin_bottom(v),
            StyleProps::InsetLeft(v) => s.inset_left(v),
            StyleProps::InsetTop(v) => s.inset_top(v),
            StyleProps::InsetRight(v) => s.inset_right(v),
            StyleProps::InsetBottom(v) => s.inset_bottom(v),
            StyleProps::ZIndex(v) => s.z_index(v),
            StyleProps::Cursor(v) => s.cursor(v),
            StyleProps::Color(v) => s.color(v),
            StyleProps::BackgroundColor(v) => s.background(v),
            StyleProps::BoxShadow(b) => s
                .box_shadow_blur(b.blur_radius)
                .box_shadow_color(b.color)
                .box_shadow_spread(b.spread)
                .box_shadow_h_offset(b.h_offset)
                .box_shadow_v_offset(b.v_offset),
            StyleProps::FontSize(v) => s.font_size(v),
            StyleProps::FontFamily(v) => s.font_family(v),
            StyleProps::FontWeight(v) => s.font_weight(v),
            StyleProps::FontStyle(v) => s.font_style(v),
            StyleProps::CursorColor(v) => s.cursor_color(v),
            StyleProps::TextOverflow(v) => s.text_overflow(v),
            StyleProps::LineHeight(v) => s.line_height(v),
            StyleProps::AspectRatio(v) => s.aspect_ratio(v),
            StyleProps::Gap(v) => s.gap(v.width),
            StyleProps::Transition((key, t)) => Self::apply_transition(&key, t, s),
        }
    }
}

#[inline]
fn parse_display(s: impl AsRef<str>) -> Result<Display, StyleError> {
    let s = s.as_ref();
    match s {
        "block" => Ok(Display::Block),
        "flex" => Ok(Display::Flex),
        "grid" => Ok(Display::Grid),
        "none" => Ok(Display::None),
        _ => Err(StyleError::new("Invalid display", s)),
    }
}

#[inline]
fn parse_justify_content(s: impl AsRef<str>) -> Result<JustifyContent, StyleError> {
    let s = s.as_ref();
    match s {
        "start" => Ok(JustifyContent::Start),
        "end" => Ok(JustifyContent::End),
        "flex-start" => Ok(JustifyContent::FlexStart),
        "flex-end" => Ok(JustifyContent::FlexEnd),
        "center" => Ok(JustifyContent::Center),
        "stretch" => Ok(JustifyContent::Stretch),
        "space-between" => Ok(JustifyContent::SpaceBetween),
        "space-evenly" => Ok(JustifyContent::SpaceEvenly),
        "space-around" => Ok(JustifyContent::SpaceAround),
        _ => Err(StyleError::new("Invalid justify-content value", s)),
    }
}

#[inline]
fn parse_align_items(s: impl AsRef<str>) -> Result<AlignItems, StyleError> {
    let s = s.as_ref();
    match s {
        "start" => Ok(AlignItems::Start),
        "end" => Ok(AlignItems::End),
        "flex-start" => Ok(AlignItems::FlexStart),
        "flex-end" => Ok(AlignItems::FlexEnd),
        "center" => Ok(AlignItems::Center),
        "baseline" => Ok(AlignItems::Baseline),
        "stretch" => Ok(AlignItems::Stretch),
        _ => Err(StyleError::new("Invalid align-items value", s)),
    }
}

#[inline]
fn parse_align_content(s: impl AsRef<str>) -> Result<AlignContent, StyleError> {
    let s = s.as_ref();
    match s {
        "start" => Ok(AlignContent::Start),
        "end" => Ok(AlignContent::End),
        "flex-start" => Ok(AlignContent::FlexStart),
        "flex-end" => Ok(AlignContent::FlexEnd),
        "center" => Ok(AlignContent::Center),
        "stretch" => Ok(AlignContent::Stretch),
        "space-between" => Ok(AlignContent::SpaceBetween),
        "space-evenly" => Ok(AlignContent::SpaceEvenly),
        "space-around" => Ok(AlignContent::SpaceAround),
        _ => Err(StyleError::new("Invalid align-content value", s)),
    }
}

#[inline]
fn parse_position(s: impl AsRef<str>) -> Result<Position, StyleError> {
    let s = s.as_ref();
    match s {
        "relative" => Ok(Position::Relative),
        "absolute" => Ok(Position::Absolute),
        _ => Err(StyleError::new("Invalid position", s)),
    }
}

#[inline]
fn parse_flex_direction(s: impl AsRef<str>) -> Result<FlexDirection, StyleError> {
    let s = s.as_ref();
    match s {
        "column" => Ok(FlexDirection::Column),
        "column-reverse" => Ok(FlexDirection::ColumnReverse),
        "row" => Ok(FlexDirection::Row),
        "row-reverse" => Ok(FlexDirection::RowReverse),
        _ => Err(StyleError::new("Invalid flex direction", s)),
    }
}

#[inline]
fn parse_flex_wrap(s: impl AsRef<str>) -> Result<FlexWrap, StyleError> {
    let s = s.as_ref();
    match s {
        "wrap" => Ok(FlexWrap::Wrap),
        "no-wrap" => Ok(FlexWrap::NoWrap),
        "wrap-reverse" => Ok(FlexWrap::WrapReverse),
        _ => Err(StyleError::new("Invalid flex wrap", s)),
    }
}

#[allow(clippy::cast_possible_truncation)]
#[inline]
fn parse_f32(s: impl AsRef<str>) -> Result<f32, StyleError> {
    let px = parse_px(s)?;
    Ok(px.0 as f32)
}

// #[inline]
// fn parse_f32_opt(s: impl AsRef<str>) -> Result<Option<f32>, StyleError> {
//     match parse_px(s) {
//         Ok(v) => Ok(Some(v.0 as f32)),
//         Err(e) => Err(e),
//     }
// }

// #[inline]
// fn parse_line_height(s: impl AsRef<str>) -> Result<LineHeightValue, StyleError> {
//     match parse_px(s) {
//         Ok(val) => Ok(LineHeightValue::Normal(val.0 as f32)),
//         Err(e) => Err(e),
//     }
// }

#[inline]
pub fn parse_px(s: impl AsRef<str>) -> Result<Px, StyleError> {
    let val = s.as_ref();

    let Some(stripped) = val.strip_suffix("px") else {
        return Err(StyleError::new("Cannot convert to Px", val));
    };

    let value_str = stripped.replace(' ', "");
    let ft = f64::from_str(&value_str).map_err(|e| StyleError::new(&e, val))?;

    Ok(Px(ft))
}

#[inline]
pub fn parse_pct(s: impl AsRef<str>) -> Result<Pct, StyleError> {
    let val = s.as_ref();

    let Some(stripped) = val.strip_suffix('%') else {
        return Err(StyleError::new("Cannot convert to Pct", val));
    };

    let value_str = stripped.replace(' ', "");
    let ft = f64::from_str(&value_str).map_err(|e| StyleError::new(&e, val))?;

    Ok(Pct(ft))
}

#[inline]
pub fn parse_px_pct(s: impl AsRef<str>) -> Result<PxPct, StyleError> {
    if let Ok(px) = parse_px(&s) {
        return Ok(PxPct::Px(px.0));
    }

    if let Ok(pct) = parse_pct(&s) {
        return Ok(PxPct::Pct(pct.0));
    }

    Err(StyleError::new("Cannot convert to PxPctAuto", s.as_ref()))
}

#[inline]
pub fn parse_pxpctauto(s: impl AsRef<str>) -> Result<PxPctAuto, StyleError> {
    let s = s.as_ref();
    if s == "auto" {
        return Ok(PxPctAuto::Auto);
    }

    match parse_px_pct(s) {
        Ok(PxPct::Px(px)) => Ok(PxPctAuto::Px(px)),
        Ok(PxPct::Pct(pct)) => Ok(PxPctAuto::Pct(pct)),
        Err(e) => Err(e),
    }
}

#[inline]
pub fn parse_color(s: impl AsRef<str>) -> Result<Color, StyleError> {
    // TODO Parse rgb strings
    let s = s.as_ref();

    if let Some(matches) = BRACKETS_REGEX.captures(s) {
        let group = matches.get(1).ok_or(StyleError::new("Invalid color value", s))?;

        if s.starts_with("rgba") {
            return parse_rgba(group.as_str());
        }

        if s.starts_with("rgb") {
            return parse_rgb(group.as_str());
        }

        if s.starts_with("hsl") {
            return Err(StyleError::new("hsl not supported", s));
        }

        if s.starts_with("hwb") {
            return Err(StyleError::new("hwb not supported", s));
        }
    }

    Color::parse(s).ok_or(StyleError::new("Invalid color value", s))
}

#[inline]
fn parse_i32(s: impl AsRef<str>) -> Result<i32, StyleError> {
    let s = s.as_ref();
    s.parse::<i32>().map_err(|e| StyleError::new(&e, s))
}

#[inline]
fn parse_cursor_style(s: impl AsRef<str>) -> Result<CursorStyle, StyleError> {
    let s = s.as_ref();
    match s {
        "default" => Ok(CursorStyle::Default),
        "pointer" => Ok(CursorStyle::Pointer),
        "text" => Ok(CursorStyle::Text),
        "col-resize" => Ok(CursorStyle::ColResize),
        "row-resize" => Ok(CursorStyle::RowResize),
        "w-resize" => Ok(CursorStyle::WResize),
        "e-resize" => Ok(CursorStyle::EResize),
        "s-resize" => Ok(CursorStyle::SResize),
        "n-resize" => Ok(CursorStyle::NResize),
        "nw-resize" => Ok(CursorStyle::NwResize),
        "ne-resize" => Ok(CursorStyle::NeResize),
        "sw-resize" => Ok(CursorStyle::SwResize),
        "se-resize" => Ok(CursorStyle::SeResize),
        "nesw-resize" => Ok(CursorStyle::NeswResize),
        "nwse-resize" => Ok(CursorStyle::NwseResize),
        _ => Err(StyleError::new("Invalid cursor style value", s)),
    }
}

#[allow(clippy::unnecessary_wraps)]
#[inline]
fn to_owned(s: impl AsRef<str>) -> Result<String, StyleError> {
    Ok(s.as_ref().to_string())
}

#[inline]
fn parse_font_weight(s: impl AsRef<str>) -> Result<Weight, StyleError> {
    let s = s.as_ref();
    match s {
        "100" | "thin" => Ok(Weight(100)),
        "200" => Ok(Weight(200)),
        "300" => Ok(Weight(300)),
        "400" | "normal" => Ok(Weight(400)),
        "500" => Ok(Weight(500)),
        "600" => Ok(Weight(600)),
        "700" | "bold" => Ok(Weight(700)),
        "800" => Ok(Weight(800)),
        "900" => Ok(Weight(900)),
        _ => Err(StyleError::new("Invalid cursor style value", s)),
    }
}

#[inline]
fn parse_font_style(s: impl AsRef<str>) -> Result<floem::cosmic_text::Style, StyleError> {
    let s = s.as_ref();
    match s {
        "normal" => Ok(floem::cosmic_text::Style::Normal),
        "italic" => Ok(floem::cosmic_text::Style::Italic),
        "oblique" => Ok(floem::cosmic_text::Style::Oblique),
        _ => Err(StyleError::new("Invalid font style value", s)),
    }
}

#[inline]
fn parse_text_overflow(s: impl AsRef<str>) -> Result<TextOverflow, StyleError> {
    let s = s.as_ref();
    match s {
        "clip" => Ok(TextOverflow::Clip),
        "ellipsis" => Ok(TextOverflow::Ellipsis),
        "wrap" => Ok(TextOverflow::Wrap),
        _ => Err(StyleError::new("Invalid text overflow value", s)),
    }
}

#[inline]
fn parse_gap(s: impl AsRef<str>) -> Result<Size<Px>, StyleError> {
    match parse_px(s) {
        Ok(val) => Ok(Size {
            width: val,
            height: val,
        }),
        Err(e) => Err(e),
    }
}

#[inline]
fn parse_box_shadow(s: impl AsRef<str>) -> Result<BoxShadow, StyleError> {
    let s = s.as_ref();

    // TODO Regex match and remove color values

    let parts = s.split_whitespace().map(str::trim).collect::<Vec<_>>();

    match &parts[..] {
        ["none"] => Ok(BoxShadow::default()),
        [a, b] => parse_box_shadow_2([a, b]),
        [a, b, c] => parse_box_shadow_3([a, b, c]),
        [a, b, c, d] => parse_box_shadow_4([a, b, c, d]),
        [a, b, c, d, e] => parse_box_shadow_5([a, b, c, d, e]),
        _ => Err(StyleError::new("invalid box-shadow value", s)),
    }
}

#[inline]
fn parse_box_shadow_2([a, b]: [&str; 2]) -> Result<BoxShadow, StyleError> {
    if let (Ok(h_offset), Ok(v_offset)) = (parse_px_pct(a), parse_px_pct(b)) {
        return Ok(BoxShadow {
            h_offset,
            v_offset,
            ..BoxShadow::default()
        });
    };

    Err(StyleError::new("Invalid box shadow value", &format!("{a} {b}")))
}

#[inline]
fn parse_box_shadow_3([a, b, c]: [&str; 3]) -> Result<BoxShadow, StyleError> {
    // <h_offset> <v_offset> <color>
    if let (Ok(h_offset), Ok(v_offset), Ok(color)) = (parse_px(a), parse_px(b), parse_color(c)) {
        return Ok(BoxShadow {
            color,
            h_offset: h_offset.into(),
            v_offset: v_offset.into(),
            ..BoxShadow::default()
        });
    }

    // <color> <h_offset> <v_offset>
    if let (Ok(color), Ok(h_offset), Ok(v_offset)) = (parse_color(a), parse_px(b), parse_px(c)) {
        return Ok(BoxShadow {
            color,
            h_offset: h_offset.into(),
            v_offset: v_offset.into(),
            ..BoxShadow::default()
        });
    }
    // <h_offset> <v_offset> <blur>
    if let (Ok(h_offset), Ok(v_offset), Ok(blur_radius)) = (parse_px(a), parse_px(b), parse_px(c)) {
        return Ok(BoxShadow {
            blur_radius: blur_radius.into(),
            h_offset: h_offset.into(),
            v_offset: v_offset.into(),
            ..BoxShadow::default()
        });
    }

    Err(StyleError::new("Invalid box-shadow value", &format!("{a} {b} {c}")))
}

#[inline]
fn parse_box_shadow_4([a, b, c, d]: [&str; 4]) -> Result<BoxShadow, StyleError> {
    // <h_offset> <v_offset> <blur_radius> <color>
    if let (Ok(h_offset), Ok(v_offset), Ok(blur_radius), Ok(color)) =
        (parse_px(a), parse_px(b), parse_px(c), parse_color(d))
    {
        return Ok(BoxShadow {
            color,
            blur_radius: blur_radius.into(),
            h_offset: h_offset.into(),
            v_offset: v_offset.into(),
            ..BoxShadow::default()
        });
    }

    // <color> <h_offset> <v_offset> <blur_radius>
    if let (Ok(color), Ok(h_offset), Ok(v_offset), Ok(blur_radius)) =
        (parse_color(a), parse_px(b), parse_px(c), parse_px(d))
    {
        return Ok(BoxShadow {
            color,
            blur_radius: blur_radius.into(),
            h_offset: h_offset.into(),
            v_offset: v_offset.into(),
            ..BoxShadow::default()
        });
    }
    // <h_offset> <v_offset> <blur_radius> <blur_spread>
    if let (Ok(h_offset), Ok(v_offset), Ok(blur_radius), Ok(blur_spread)) =
        (parse_px(a), parse_px(b), parse_px(c), parse_px(d))
    {
        return Ok(BoxShadow {
            blur_radius: blur_radius.into(),
            spread: blur_spread.into(),
            h_offset: h_offset.into(),
            v_offset: v_offset.into(),
            ..BoxShadow::default()
        });
    }

    Err(StyleError::new("Invalid box-shadow value", &format!("{a} {b} {c} {d}")))
}

#[inline]
fn parse_box_shadow_5([a, b, c, d, e]: [&str; 5]) -> Result<BoxShadow, StyleError> {
    // <h_offset> <v_offset> <blur_radius> <blur_spread> <color>
    if let (Ok(h_offset), Ok(v_offset), Ok(blur_radius), Ok(blur_spread), Ok(color)) =
        (parse_px(a), parse_px(b), parse_px(c), parse_px(d), parse_color(e))
    {
        return Ok(BoxShadow {
            h_offset: h_offset.into(),
            v_offset: v_offset.into(),
            blur_radius: blur_radius.into(),
            spread: blur_spread.into(),
            color,
        });
    }

    // <color> <h_offset> <v_offset> <blur_radius> <blur_spread>
    if let (Ok(color), Ok(h_offset), Ok(v_offset), Ok(blur_radius), Ok(blur_spread)) =
        (parse_color(a), parse_px(b), parse_px(c), parse_px(d), parse_px(e))
    {
        return Ok(BoxShadow {
            h_offset: h_offset.into(),
            v_offset: v_offset.into(),
            blur_radius: blur_radius.into(),
            spread: blur_spread.into(),
            color,
        });
    }

    Err(StyleError::new(
        "Invalid box-shadow value",
        &format!("{a} {b} {c} {d} {e}"),
    ))
}

#[inline]
fn parse_rgba(s: impl AsRef<str>) -> Result<Color, StyleError> {
    let s = s.as_ref();
    let parts = s.split(',').map(str::trim).collect::<Vec<_>>();

    if let [r, g, b, a] = parts[..] {
        if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
            parse_rgb_value(r),
            parse_rgb_value(g),
            parse_rgb_value(b),
            parse_rgb_alpha(a),
        ) {
            return Ok(Color::rgba8(r, g, b, a));
        }
    }

    Err(StyleError::new("Invalid rgba value", s))
}

#[inline]
fn parse_rgb(s: impl AsRef<str>) -> Result<Color, StyleError> {
    let s = s.as_ref();
    let parts = s.split(',').map(str::trim).collect::<Vec<_>>();

    if let [r, g, b] = parts[..] {
        if let (Ok(r), Ok(g), Ok(b)) = (parse_rgb_value(r), parse_rgb_value(g), parse_rgb_value(b)) {
            return Ok(Color::rgb8(r, g, b));
        }
    }

    Err(StyleError::new("Invalid rgb value", s))
}

#[inline]
fn parse_rgb_value(s: &str) -> Result<u8, StyleError> {
    if let Some(stripped) = s.strip_suffix('%') {
        stripped
            .parse::<u8>()
            .map_or_else(|e| Err(StyleError::new(&e, s)), |v| Ok(v.clamp(0, 100)))
    } else {
        s.parse::<u8>().map_or_else(|e| Err(StyleError::new(&e, s)), Ok)
    }
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
#[inline]
fn parse_rgb_alpha(s: &str) -> Result<u8, StyleError> {
    s.parse::<f64>().map_or_else(
        |e| Err(StyleError::new(&e, s)),
        |v| Ok((v.clamp(0.0, 1.0) * 255.0) as u8),
    )
}

#[inline]
fn parse_transition(s: impl AsRef<str>) -> Result<(String, Transition), StyleError> {
    let s = s.as_ref();
    let mut parts = s.split_whitespace().map(str::trim);

    let Some(key) = parts.next() else {
        return Err(StyleError::new("Missing transition key", s));
    };

    let Some(duration) = parts.next() else {
        return Err(StyleError::new("Missing transition duration", s));
    };

    let df = parse_seconds(duration)?;

    let t = Transition::linear(df);

    Ok((key.to_string(), t))
}

#[inline]
fn parse_seconds(s: &str) -> Result<f64, StyleError> {
    let Some(stripped) = s.strip_suffix('s') else {
        return Err(StyleError::new("Duration must be given as seconds i.e. 1s or 0.1s", s));
    };

    let f = stripped.parse::<f64>().map_err(|e| StyleError::new(&e, s))?;

    Ok(f)
}
