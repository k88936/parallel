.PHONY: binding

binding:
	cd crates/common && TS_RS_LARGE_INT=number  TS_RS_EXPORT_DIR="../../parallel-web/src/types" cargo test export_bindings