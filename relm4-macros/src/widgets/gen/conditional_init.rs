use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Ident, Path};

use crate::widgets::{
    AssignProperty, AssignPropertyAttr, ConditionalBranches, ConditionalWidget, MatchArm,
    Properties, Property, PropertyName, PropertyType, ReturnedWidget, Widget,
};

impl Property {
    fn conditional_init_stream(
        &self,
        stream: &mut TokenStream2,
        w_name: &Ident,
        model_name: &Ident,
        is_conditional: bool,
        relm4_path: &Path,
    ) {
        match &self.ty {
            PropertyType::Assign(assign) => assign.conditional_init_stream(
                stream,
                &self.name,
                w_name,
                model_name,
                is_conditional,
                relm4_path,
            ),
            PropertyType::Widget(widget) => {
                widget.conditional_init_stream(stream, model_name, is_conditional, relm4_path)
            }
            PropertyType::ConditionalWidget(cond_widget) => {
                cond_widget.conditional_init_stream(stream, model_name, relm4_path)
            }
            PropertyType::SignalHandler(_) | PropertyType::ParseError(_) => (),
        }
    }
}

impl Properties {
    fn conditional_init_stream(
        &self,
        stream: &mut TokenStream2,
        w_name: &Ident,
        model_name: &Ident,
        is_conditional: bool,
        relm4_path: &Path,
    ) {
        for prop in &self.properties {
            prop.conditional_init_stream(stream, w_name, model_name, is_conditional, relm4_path);
        }
    }
}

impl Widget {
    pub fn init_conditional_init_stream(
        &self,
        stream: &mut TokenStream2,
        model_name: &Ident,
        relm4_path: &Path,
    ) {
        self.conditional_init_stream(stream, model_name, false, relm4_path);
    }

    fn conditional_init_stream(
        &self,
        stream: &mut TokenStream2,
        model_name: &Ident,
        is_conditional: bool,
        relm4_path: &Path,
    ) {
        let w_name = &self.name;
        self.properties.conditional_init_stream(
            stream,
            w_name,
            model_name,
            is_conditional,
            relm4_path,
        );
        if let Some(returned_widget) = &self.returned_widget {
            returned_widget.conditional_init_stream(stream, model_name, is_conditional, relm4_path);
        }
    }
}

impl ConditionalWidget {
    fn conditional_init_stream(
        &self,
        stream: &mut TokenStream2,
        model_name: &Ident,
        relm4_path: &Path,
    ) {
        let brach_stream = match &self.branches {
            ConditionalBranches::If(if_branches) => {
                let mut stream = TokenStream2::new();

                for (index, branch) in if_branches.iter().enumerate() {
                    let mut inner_update_stream = TokenStream2::new();
                    branch.widget.conditional_init_stream(
                        &mut inner_update_stream,
                        model_name,
                        true,
                        relm4_path,
                    );
                    branch.update_stream(&mut stream, inner_update_stream, index);
                }
                stream
            }
            ConditionalBranches::Match((match_token, expr, match_arms)) => {
                let mut inner_tokens = TokenStream2::new();

                for (index, match_arm) in match_arms.iter().enumerate() {
                    let mut inner_update_stream = TokenStream2::new();
                    match_arm.widget.conditional_init_stream(
                        &mut inner_update_stream,
                        model_name,
                        true,
                        relm4_path,
                    );

                    let MatchArm {
                        pattern,
                        guard,
                        arrow,
                        ..
                    } = match_arm;

                    let (guard_if, guard_expr) = if let Some((guard_if, guard_expr)) = guard {
                        (Some(guard_if), Some(guard_expr))
                    } else {
                        (None, None)
                    };

                    let index = index.to_string();
                    inner_tokens.extend(quote! {
                        #pattern #guard_if #guard_expr #arrow {
                            #inner_update_stream
                            #index
                        },
                    })
                }
                quote! {
                    #match_token #expr {
                        #inner_tokens
                    }
                }
            }
        };

        let w_name = &self.name;
        stream.extend(quote! {
            let __current_page = "";
            #w_name.set_visible_child_name(#brach_stream);
        });
    }
}

impl ReturnedWidget {
    fn conditional_init_stream(
        &self,
        stream: &mut TokenStream2,
        model_name: &Ident,
        is_conditional: bool,
        relm4_path: &Path,
    ) {
        let w_name = &self.name;
        self.properties.conditional_init_stream(
            stream,
            w_name,
            model_name,
            is_conditional,
            relm4_path,
        );
    }
}

impl AssignProperty {
    fn conditional_init_stream(
        &self,
        stream: &mut TokenStream2,
        p_name: &PropertyName,
        w_name: &Ident,
        model_name: &Ident,
        is_conditional: bool,
        relm4_path: &Path,
    ) {
        // Unconditional code is handled in the "normal" init stream
        if is_conditional {
            match &self.attr {
                AssignPropertyAttr::None => (),
                AssignPropertyAttr::Watch => {
                    self.assign_stream(stream, p_name, w_name, relm4_path);
                }
                AssignPropertyAttr::Track((track_stream, paste_model)) => {
                    let mut assign_stream = TokenStream2::new();
                    self.assign_stream(&mut assign_stream, p_name, w_name, relm4_path);
                    let model = paste_model.then(|| model_name);

                    stream.extend(quote_spanned! {
                        track_stream.span() => if #model #track_stream {
                            #assign_stream
                        }
                    });
                }
            }
        }
    }
}