// This file is part of Gear.

// Copyright (C) 2021-2024 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Provides macros for async runtime of Gear programs.

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = syn::parse_macro_input!(item as syn::ItemFn);
    let ident = &function.sig.ident;
    let extern_ident = Ident::new(&format!("test_{}", ident), Span::call_site());

    quote! {
        #function

        #[no_mangle]
        pub unsafe extern "C" fn #extern_ident() {
            let test_future = async {
                let context = gear_test_runtime::TestContext::current();
                let test_name = stringify!(#ident);
                context.test_start(test_name);

                match #ident(&context).await {
                    gear_test_runtime::TestResult::Ok => {
                        context.test_success(test_name);
                    },
                    gear_test_runtime::TestResult::Fail(hint) => {
                        context.test_fail(test_name, hint);
                    }
                };
            }.boxed();

            gear_test_runtime::CONTEXT_FUTURES.push(test_future);
        }
    }
    .into()
}
