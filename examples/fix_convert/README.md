Debug `Convert` Meta
### Build and Run
Init
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
Error the trait bound `RecordV2: Convert<_>` is not satisfied
```
try to convert RecordV2 from RecordV1 but trait bound not satisfied, try add metas for object

```
@@ -57,7 +57,7 @@ objects:
         ty: UnsignedInteger
   - ty: RecordV2
     metas:
-      - derives: [Clone, Debug]
+      - derives: [Clone, Debug, Convert]
     fields:
       - name: id
         ty: String
``` 
Build again, and see different error
```
Error meta prefix 'convert.input' not found at 'RecordV2'
```
need to specify input to convert from
```
@@ -58,6 +58,8 @@ objects:
   - ty: RecordV2
     metas:
       - derives: [Clone, Debug, Convert]
+      - convert:
+          input: RecordV1
     fields:
       - name: id
         ty: String
```
build again, and see different error ...
```
Error meta prefix 'convert.from' not found at 'RecordV2.id'
```
need to specify field mapping
```
     fields:
       - name: id
         ty: String
-      - name: amount
+        metas:
+          - convert:
+              from: key
+      - name: amount
         ty: UnsignedInteger
+        metas:
+          - convert:
+              from: value.amount
```  
build and succeed