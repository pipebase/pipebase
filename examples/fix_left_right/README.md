Debug `LeftRight` Meta
### Build and Run
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -r -d
```
error
```
Error the trait bound `debug_io::Record: LeftRight` is not satisfied
```
`RedisStringsWriter` requires input implement `LeftRight` trait, fix it
```
@@ -41,9 +41,13 @@ pipes:
 objects:
   - ty: Record
     metas:
-      - derives: [Clone, Debug, Deserialize]
+      - derives: [Clone, Debug, Deserialize, LeftRight]
     fields:
       - name: key
+        metas:
+          - tag: Left
         ty: String
       - name: value
+        metas:
+          - tag: Right
         ty: UnsignedInteger
``` 