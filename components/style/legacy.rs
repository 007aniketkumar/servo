/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Legacy presentational attributes defined in the HTML5 specification: `<td width>`,
//! `<input size>`, and so forth.

use node::{TElement, TElementAttributes, TNode};
use properties::{SpecifiedValue, WidthDeclaration, specified};
use selector_matching::{DeclarationBlock, Stylist};

use servo_util::geometry::Au;
use servo_util::smallvec::VecLike;
use servo_util::str::{AutoLpa, LengthLpa, PercentageLpa};

/// Legacy presentational attributes that take a length as defined in HTML5 § 2.4.4.4.
pub enum LengthAttribute {
    /// `<td width>`
    WidthLengthAttribute,
}

/// Legacy presentational attributes that take an integer as defined in HTML5 § 2.4.4.2.
pub enum IntegerAttribute {
    /// `<input size>`
    SizeIntegerAttribute,
}

/// Extension methods for `Stylist` that cause rules to be synthesized for legacy attributes.
pub trait PresentationalHintSynthesis {
    /// Synthesizes rules from various HTML attributes (mostly legacy junk from HTML4) that confer
    /// *presentational hints* as defined in the HTML5 specification. This handles stuff like
    /// `<body bgcolor>`, `<input size>`, `<td width>`, and so forth.
    fn synthesize_presentational_hints_for_legacy_attributes<'a,E,N,V>(
                                                             &self,
                                                             node: &N,
                                                             matching_rules_list: &mut V,
                                                             shareable: &mut bool)
                                                             where E: TElement<'a> +
                                                                      TElementAttributes,
                                                                   N: TNode<'a,E>,
                                                                   V: VecLike<DeclarationBlock>;
}

impl PresentationalHintSynthesis for Stylist {
    fn synthesize_presentational_hints_for_legacy_attributes<'a,E,N,V>(
                                                             &self,
                                                             node: &N,
                                                             matching_rules_list: &mut V,
                                                             shareable: &mut bool)
                                                             where E: TElement<'a> +
                                                                      TElementAttributes,
                                                                   N: TNode<'a,E>,
                                                                   V: VecLike<DeclarationBlock> {
        let element = node.as_element();
        match element.get_local_name() {
            name if *name == atom!("td") => {
                match element.get_length_attribute(WidthLengthAttribute) {
                    AutoLpa => {}
                    PercentageLpa(percentage) => {
                        let width_value = specified::LPA_Percentage(percentage);
                        matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                                WidthDeclaration(SpecifiedValue(width_value))));
                        *shareable = false
                    }
                    LengthLpa(length) => {
                        let width_value = specified::LPA_Length(specified::Au_(length));
                        matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                                WidthDeclaration(SpecifiedValue(width_value))));
                        *shareable = false
                    }
                };
            }
            name if *name == atom!("input") => {
                match element.get_integer_attribute(SizeIntegerAttribute) {
                    Some(value) if value != 0 => {
                        // Per HTML 4.01 § 17.4, this value is in characters if `type` is `text` or
                        // `password` and in pixels otherwise.
                        //
                        // FIXME(pcwalton): More use of atoms, please!
                        let value = match element.get_attr(&ns!(""), &atom!("type")) {
                            Some("text") | Some("password") => {
                                specified::ServoCharacterWidth(value)
                            }
                            _ => specified::Au_(Au::from_px(value as int)),
                        };
                        matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                                WidthDeclaration(SpecifiedValue(specified::LPA_Length(
                                            value)))));
                        *shareable = false
                    }
                    Some(_) | None => {}
                }
            }
            _ => {}
        }
    }
}

