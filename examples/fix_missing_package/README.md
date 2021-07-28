Debug missing package 
### Build and Run
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o fix_it -r -d
```
Error
```
Error error: cannot find derive macro `Deserialize` in this scope
Error error[E0277]: the trait bound `for<'de> fix_missing_package::Record: serde::de::Deserialize<'de>` is not satisfied
```
`JsonDeser` requires input implements `Deserialize` trait, we need to include crate [`serde`] as app dependency, fix it
```
@@ -9,6 +9,10 @@ dependencies:
   - package: pipejson
     path: ../../../pipeware/pipejson
     modules: ["pipejson::*"]
+  - package: serde
+    version: "1.0"
+    features: ["derive"]
+    modules: ["serde::Deserialize"]
``` 

[`serde`]: https://docs.serde.rs/serde/index.html