//! Initialize default keymap from config
//!

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use rmk_config::toml_config::{LayoutConfig, MatrixConfig};

/// Read the default keymap setting in `keyboard.toml` and add as a `static KEYMAP`
pub(crate) fn expand_layout_init(
    layout_config: Option<LayoutConfig>,
    matrix_config: MatrixConfig,
) -> TokenStream2 {
    if let Some(l) = layout_config {
        // Check the size of layers, rows and cols first
        if let Err(err_msg) = check_keymap_size(&l, matrix_config) {
            return syn::Error::new_spanned::<TokenStream2, String>(quote! {}, err_msg)
                .to_compile_error()
                .into();
        }

        let mut layers = vec![];
        for layer in l.keymap {
            layers.push(expand_layer(layer));
        }
        return quote! { static KEYMAP: [[[::rmk::action::KeyAction; COL]; ROW]; NUM_LAYER] = [#(#layers), *]; };
    };
    quote! {}
}

/// Push rows in the layer
fn expand_layer(layer: Vec<Vec<String>>) -> TokenStream2 {
    let mut rows = vec![];
    for row in layer {
        rows.push(expand_row(row));
    }
    quote! { [#(#rows), *] }
}

/// Push keys in the row 
fn expand_row(row: Vec<String>) -> TokenStream2 {
    let mut keys = vec![];
    for key in row {
        keys.push(parse_key(key));
    }
    quote! { [#(#keys), *] }
}

/// Check whether the size of keymap matches matrix config
fn check_keymap_size(l: &LayoutConfig, matrix_config: MatrixConfig) -> Result<(), String> {
    // Layer
    let layer_num = l.keymap.len();
    if layer_num as u8 != matrix_config.layers {
        return Err(
            "keyboard.toml: Layer number in keymap doesn't match with [matrix.layers]".to_string(),
        );
    }
    // Row
    if let Some(_) = l
        .keymap
        .iter()
        .map(|r| r.len())
        .find(|l| *l as u8 != matrix_config.rows)
    {
        return Err(
            "keyboard.toml: Row number in keymap doesn't match with [matrix.row]".to_string(),
        );
    }
    // Col
    if let Some(_) = l
        .keymap
        .iter()
        .filter_map(|r| {
            r.iter()
                .map(|c| c.len())
                .find(|l| *l as u8 != matrix_config.cols)
        })
        .next()
    {
        // Find a row whose col num is wrong
        return Err(
            "keyboard.toml: Col number in keymap doesn't match with [matrix.col]".to_string(),
        );
    }
    Ok(())
}

/// Parse the key string at a single position
fn parse_key(key: String) -> TokenStream2 {
    if key.len() < 5 {
        return if key.len() > 0 && key.trim_start_matches("_").len() == 0 {
            quote! { ::rmk::a!(No) }
        } else {
            let ident = format_ident!("{}", key);
            quote! { ::rmk::k!(#ident) }
        };
    }
    match &key[0..3] {
        "MO(" => {
            let layer = get_layer(key, "MO(", ")");
            quote! {
                ::rmk::mo!(#layer)
            }
        }
        "OSL" => {
            let layer = get_layer(key, "OSL(", ")");
            quote! {
                ::rmk::osl!(#layer)
            }
        }
        "LM(" => {
            if let Some(internal) = key.trim_start_matches("LM(").strip_suffix(")") {
                let keys: Vec<&str> = internal
                    .split_terminator(",")
                    .map(|w| w.trim())
                    .filter(|w| w.len() > 0)
                    .collect();
                if keys.len() != 2 {
                    return quote! {
                        compile_error!("keyboard.toml: LM(layer, modifier) invalid, please check the documentation: https://haobogu.github.io/rmk/configuration.html");
                    };
                }
                let layer = keys[0].parse::<u8>().unwrap();

                // Get modifier combination, in types of mod1 | mod2 | ...
                let mut right = false;
                let mut gui = false;
                let mut alt = false;
                let mut shift = false;
                let mut ctrl = false;
                keys[1].split_terminator("|").for_each(|w| {
                    let w = w.trim();
                    match w {
                        "LShift" => shift = true,
                        "LCtrl" => ctrl = true,
                        "LAlt" => alt = true,
                        "Lgui" => gui = true,
                        "RShift" => {
                            right = true;
                            shift = true;
                        }
                        "RCtrl" => {
                            right = true;
                            ctrl = true;
                        }
                        "RAlt" => {
                            right = true;
                            alt = true;
                        }
                        "Rgui" => {
                            right = true;
                            gui = true;
                        }
                        _ => (),
                    }
                });

                if (gui || alt || shift || ctrl) == false {
                    return quote! {
                        compile_error!("keyboard.toml: modifier in LM(layer, modifier) is not valid! Please check the documentation: https://haobogu.github.io/rmk/configuration.html");
                    };
                }
                quote! {
                    ::rmk::lm!(#layer, ::rmk::keycode::ModifierCombination::new_from(#right, #gui, #alt, #shift, #ctrl))
                }
            } else {
                return quote! {
                    compile_error!("keyboard.toml: LM(layer, modifier) invalid, please check the documentation: https://haobogu.github.io/rmk/configuration.html");
                };
            }
        }
        "LT(" => {
            let keys: Vec<&str> = key
                .trim_start_matches("LT(")
                .trim_end_matches(")")
                .split_terminator(",")
                .map(|w| w.trim())
                .filter(|w| w.len() > 0)
                .collect();
            if keys.len() != 2 {
                return quote! {
                    compile_error!("keyboard.toml: LT(layer, key) invalid, please check the documentation: https://haobogu.github.io/rmk/configuration.html");
                };
            }
            let layer = keys[0].parse::<u8>().unwrap();
            let key = format_ident!("{}", keys[1]);
            quote! {
                ::rmk::lt!(#layer, #key)
            }
        }
        "TT(" => {
            let layer = get_layer(key, "TT(", ")");
            quote! {
                ::rmk::tt!(#layer)
            }
        }
        "TG(" => {
            let layer = get_layer(key, "TG(", ")");
            quote! {
                ::rmk::tg!(#layer)
            }
        }
        _ => {
            let ident = format_ident!("{}", key);
            quote! {::rmk::k!(#ident) }
        }
    }
}

/// Parse the string literal like `MO(1)`, `OSL(1)`, get the layer number in it.
/// The caller should pass the trimmed prefix and suffix
fn get_layer(key: String, prefix: &str, suffix: &str) -> u8 {
    let layer_str = key.trim_start_matches(prefix).trim_end_matches(suffix);
    layer_str.parse::<u8>().unwrap()
}
