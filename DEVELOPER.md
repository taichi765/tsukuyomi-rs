## Coding Guidelines
Avoid using `in-out property` if there's another way.  
Rust code and UI communicate via global singletons named `XxxStore`.  
Small components should not access Store directory. Use `in property` instead.  

## Directory structure
- `containers/` are components to handle Store and pass properties to children views.
- `view/` are components to represent object.
- `parts/` are small and reusable components.
