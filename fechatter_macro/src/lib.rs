//! # `fechatter_macro`
//!
//! This crate provides the `#[middleware_builder]` procedural macro that generates a
//! **type-state** builder for composing Axum middleware layers.
//!
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, Ident, LitStr, Path, Result, Token, parenthesized};

// Represents a single state transition rule:
// state(CurrentStateIdent) -> method_ident uses "actual::middleware::path" => NewStateIdent
#[derive(Debug)]
struct StateTransitionRule {
  current_state: Ident,
  _arrow_token: Token![->],
  method_name: Ident,
  _uses_keyword: Ident,
  middleware_path: Path,
  _next_state_arrow_token: Token![=>],
  new_state: Ident,
  // New fields for more powerful configuration
  required_traits: Option<Path>,
  return_type: Option<Path>,
}

impl Parse for StateTransitionRule {
  fn parse(input: ParseStream) -> Result<Self> {
    let state_kw: Ident = input.parse()?;
    if state_kw != "state" {
      return Err(syn::Error::new(state_kw.span(), "expected `state` keyword"));
    }
    let content_current_state;
    parenthesized!(content_current_state in input);
    let current_state: Ident = content_current_state.parse()?;
    let arrow_token: Token![->] = input.parse()?;
    let method_name: Ident = input.parse()?;

    // Expect: uses "actual::middleware::path"
    let uses_keyword: Ident = input.parse()?;
    if uses_keyword != "uses" {
      return Err(syn::Error::new(
        uses_keyword.span(),
        "expected `uses` keyword after method name",
      ));
    }
    let middleware_path_str: LitStr = input.parse()?;
    let middleware_path: Path = middleware_path_str.parse_with(syn::Path::parse_mod_style)?;

    let next_state_arrow_token: Token![=>] = input.parse()?;
    let new_state: Ident = input.parse()?;

    // Optional additional configuration
    let mut required_traits = None;
    let mut return_type = None;

    // Parse optional "requires" keyword
    if input.peek(Ident) && input.parse::<Ident>().unwrap() == "requires" {
      let traits_str: LitStr = input.parse()?;
      required_traits = Some(traits_str.parse_with(syn::Path::parse_mod_style)?);
    }

    // Parse optional "returns" keyword
    if input.peek(Ident) && input.parse::<Ident>().unwrap() == "returns" {
      let return_type_str: LitStr = input.parse()?;
      return_type = Some(return_type_str.parse_with(syn::Path::parse_mod_style)?);
    }

    Ok(StateTransitionRule {
      current_state,
      _arrow_token: arrow_token,
      method_name,
      _uses_keyword: uses_keyword,
      middleware_path,
      _next_state_arrow_token: next_state_arrow_token,
      new_state,
      required_traits,
      return_type,
    })
  }
}

#[derive(Debug)]
struct AllRules {
  rules: Vec<StateTransitionRule>,
  // New fields for configuration
  initial_state: Option<Ident>,
  helper_functions: bool,
  router_ext: bool,
}

impl Parse for AllRules {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut rules = Vec::new();
    let mut initial_state = None;
    let mut helper_functions = true;
    let mut router_ext = true;
    
    // Parse optional configuration
    if input.peek(Ident) && input.parse::<Ident>().unwrap() == "config" {
      let content_config;
      parenthesized!(content_config in input);
      
      while !content_config.is_empty() {
        let key: Ident = content_config.parse()?;
        content_config.parse::<Token![=]>()?;
        
        if key == "initial_state" {
          initial_state = Some(content_config.parse()?);
        } else if key == "helper_functions" {
          helper_functions = content_config.parse::<syn::LitBool>()?.value;
        } else if key == "router_ext" {
          router_ext = content_config.parse::<syn::LitBool>()?.value;
        }
        
        if content_config.peek(Token![,]) {
          content_config.parse::<Token![,]>()?;
        }
      }
      
      if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
      }
    }
    
    // Parse rules
    while !input.is_empty() {
      rules.push(input.parse::<StateTransitionRule>()?);
      if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
      } else if !input.is_empty() {
        break;
      }
    }
    
    Ok(AllRules { 
      rules,
      initial_state,
      helper_functions,
      router_ext,
    })
  }
}

#[proc_macro_attribute]
pub fn middleware_builder(args: TokenStream, input: TokenStream) -> TokenStream {
  let rules_ast = match syn::parse::<AllRules>(args) {
    Ok(parsed_rules) => parsed_rules,
    Err(e) => return TokenStream::from(e.to_compile_error()),
  };

  let input_item = match syn::parse::<DeriveInput>(input) {
    Ok(item) => item,
    Err(e) => return TokenStream::from(e.to_compile_error()),
  };

  let builder_name = &input_item.ident;
  let visibility = &input_item.vis;

  // Collect all unique state names to generate marker structs
  let mut state_names = HashSet::new();
  for rule in &rules_ast.rules {
    state_names.insert(rule.current_state.clone());
    state_names.insert(rule.new_state.clone());
  }

  // Determine the initial state, either from config or first rule's current_state
  let initial_state_marker = rules_ast.initial_state.clone().unwrap_or_else(|| {
    rules_ast
      .rules
      .get(0)
      .map(|rule| rule.current_state.clone())
      .unwrap_or_else(|| format_ident!("Initial"))
  });

  // Check if we need to define the initial state
  let needs_initial_state_def = !state_names.contains(&initial_state_marker);
  
  // Add initial state to the collection if not already present
  if needs_initial_state_def {
    state_names.insert(initial_state_marker.clone());
  }
  
  // Now create the actual quote tokens after all state collection is done
  let state_marker_structs = state_names.iter().map(|state_name| {
    quote! {
      #visibility struct #state_name;
    }
  });
  
  // Define the default initial state quote
  let default_initial_state_def = if needs_initial_state_def {
    quote! { #visibility struct #initial_state_marker; }
  } else {
    quote! {}
  };

  // Build a directed graph of states for analyzing pathways
  let mut state_graph: HashMap<Ident, Vec<(Ident, Ident)>> = HashMap::new();
  for rule in &rules_ast.rules {
    let from_state = rule.current_state.clone();
    let to_state = rule.new_state.clone();
    let method = rule.method_name.clone();
    
    state_graph
      .entry(from_state)
      .or_insert_with(Vec::new)
      .push((to_state, method));
  }

  // Generate builder struct with state type parameter
  let generated_builder_struct = quote! {
    #visibility struct #builder_name<S, T, StateType = #initial_state_marker> {
      router: axum::Router<S>,
      state: T,
      _state_marker: std::marker::PhantomData<StateType>,
    }
  };

  // Generate constructor impl for initial state
  let constructor_impl = quote! {
    impl<S, T> #builder_name<S, T, #initial_state_marker>
    where
      S: Clone + Send + Sync + 'static,
      // Base constraints for T
      T: Clone + Send + Sync + 'static,
    {
      #visibility fn new(router: axum::Router<S>, state: T) -> Self {
        Self {
          router,
          state,
          _state_marker: std::marker::PhantomData::<#initial_state_marker>,
        }
      }
      
      // Always provide build() method
      #visibility fn build(self) -> axum::Router<S> {
        self.router
      }
    }
  };

  // Generate helper functions if requested
  let helper_functions = if rules_ast.helper_functions {
    let middleware_helpers = rules_ast.rules.iter().map(|rule| {
      let method_name = &rule.method_name;
      let helper_fn_name = format_ident!("add_{}_middleware", method_name.to_string().strip_prefix("with_").unwrap_or(&method_name.to_string()));
      let middleware_path = &rule.middleware_path;
      
      quote! {
        #visibility fn #helper_fn_name<S>(router: axum::Router<S>, state: T) -> axum::Router<S>
        where
          S: Clone + Send + Sync + 'static,
          T: Clone + Send + Sync + 'static,
        {
          use axum::middleware::from_fn;
          
          router.layer(from_fn(move |req: axum::extract::Request<axum::body::Body>, next: axum::middleware::Next| {
            let state_clone = state.clone();
            async move { #middleware_path(axum::extract::State(state_clone), req, next).await }
          }))
        }
      }
    });
    
    quote! { 
      #(#middleware_helpers)*
    }
  } else {
    quote! {}
  };

  // Generate impl blocks for all state transitions
  let mut transition_impls = Vec::new();
  transition_impls.push(constructor_impl);
  
  // Group transitions by current state
  let mut transitions_by_state: HashMap<Ident, Vec<&StateTransitionRule>> = HashMap::new();
  for rule in &rules_ast.rules {
    transitions_by_state
      .entry(rule.current_state.clone())
      .or_insert_with(Vec::new)
      .push(rule);
  }
  
  // Generate impl blocks for each state with its transitions
  for (state, transitions) in transitions_by_state {
    // Generate methods for this state
    let transition_methods = transitions.iter().map(|rule| {
      let method_name = &rule.method_name;
      let new_state = &rule.new_state;
      let middleware_path = &rule.middleware_path;
      
      quote! {
        #visibility fn #method_name(self) -> #builder_name<S, T, #new_state> {
          // Use the helper function to add the middleware
          let router = #middleware_path(self.router, self.state.clone());
          
          #builder_name {
            router,
            state: self.state,
            _state_marker: std::marker::PhantomData::<#new_state>,
          }
        }
      }
    });
    
    // Generate impl block for this state
    let impl_block = quote! {
      impl<S, T> #builder_name<S, T, #state>
      where
        S: Clone + Send + Sync + 'static,
        T: Clone + Send + Sync + 'static,
      {
        #(#transition_methods)*
        
        // Always provide build() method
        #visibility fn build(self) -> axum::Router<S> {
          self.router
        }
      }
    };
    
    transition_impls.push(impl_block);
  }

  // Generate RouterExt trait if requested
  let router_ext_impl = if rules_ast.router_ext {
    quote! {
      // Extension trait for Router
      #visibility trait RouterExt<S, T> {
        fn with_middlewares(
          self,
          state: T,
        ) -> #builder_name<S, T, #initial_state_marker>;
      }
      
      impl<S, T> RouterExt<S, T> for axum::Router<S>
      where
        // Base router constraints
        S: Clone + Send + Sync + 'static,
        // Token and service provider constraints
        T: Clone + Send + Sync + 'static,
      {
        fn with_middlewares(
          self,
          state: T,
        ) -> #builder_name<S, T, #initial_state_marker> {
          #builder_name::new(self, state)
        }
      }
    }
  } else {
    quote! {}
  };

  // Combine all generated code
  let final_code = quote! {
    #(#state_marker_structs)*
    #default_initial_state_def

    #generated_builder_struct

    #helper_functions
    
    #(#transition_impls)*
    
    #router_ext_impl
  };

  TokenStream::from(final_code)
}