use burn_import::onnx::{ModelGen, RecordType};

fn main() {
    // Re-run this build script if the onnx-tests directory changes.
    println!("cargo:rerun-if-changed=tests");

    // Add onnx models.
    ModelGen::new()
        .input("tests/add/add_int.onnx")
        .input("tests/add/add.onnx")
        .input("tests/avg_pool2d/avg_pool2d.onnx")
        .input("tests/batch_norm/batch_norm.onnx")
        .input("tests/cast/cast.onnx")
        .input("tests/clip/clip_opset16.onnx")
        .input("tests/clip/clip_opset7.onnx")
        .input("tests/concat/concat.onnx")
        .input("tests/conv1d/conv1d.onnx")
        .input("tests/conv2d/conv2d.onnx")
        .input("tests/cos/cos.onnx")
        .input("tests/div/div.onnx")
        .input("tests/dropout/dropout_opset16.onnx")
        .input("tests/dropout/dropout_opset7.onnx")
        .input("tests/equal/equal.onnx")
        .input("tests/erf/erf.onnx")
        .input("tests/exp/exp.onnx")
        .input("tests/flatten/flatten.onnx")
        .input("tests/gather/gather.onnx")
        .input("tests/gelu/gelu.onnx")
        .input("tests/global_avr_pool/global_avr_pool.onnx")
        .input("tests/layer_norm/layer_norm.onnx")
        .input("tests/linear/linear.onnx")
        .input("tests/log_softmax/log_softmax.onnx")
        .input("tests/log/log.onnx")
        .input("tests/matmul/matmul.onnx")
        .input("tests/maxpool2d/maxpool2d.onnx")
        .input("tests/mul/mul.onnx")
        .input("tests/neg/neg.onnx")
        .input("tests/not/not.onnx")
        .input("tests/recip/recip.onnx")
        .input("tests/relu/relu.onnx")
        .input("tests/leaky_relu/leaky_relu.onnx")
        .input("tests/reduce_max/reduce_max.onnx")
        .input("tests/reduce_mean/reduce_mean.onnx")
        .input("tests/reshape/reshape.onnx")
        .input("tests/shape/shape.onnx")
        .input("tests/sigmoid/sigmoid.onnx")
        .input("tests/sign/sign.onnx")
        .input("tests/sin/sin.onnx")
        .input("tests/softmax/softmax.onnx")
        .input("tests/sqrt/sqrt.onnx")
        .input("tests/sub/sub_int.onnx")
        .input("tests/sub/sub.onnx")
        .input("tests/tanh/tanh.onnx")
        .input("tests/transpose/transpose.onnx")
        .input("tests/conv_transpose2d/conv_transpose2d.onnx")
        .input("tests/pow/pow.onnx")
        .input("tests/pow/pow_int.onnx")
        .input("tests/unsqueeze/unsqueeze.onnx")
        .input("tests/unsqueeze/unsqueeze_opset16.onnx")
        .input("tests/unsqueeze/unsqueeze_opset11.onnx")
        .input("tests/mask_where/mask_where.onnx")
        .out_dir("model/")
        .run_from_script();

    // The following tests are used to generate the model with different record types.
    // (e.g. bincode, pretty_json, etc.) Do not need to add new tests here, just use the default
    // record type to the ModelGen::new() call above.

    ModelGen::new()
        .input("tests/conv1d/conv1d.onnx")
        .out_dir("model/named_mpk/")
        .record_type(RecordType::NamedMpk)
        .run_from_script();

    ModelGen::new()
        .input("tests/conv1d/conv1d.onnx")
        .out_dir("model/named_mpk_half/")
        .record_type(RecordType::NamedMpk)
        .half_precision(true)
        .run_from_script();

    ModelGen::new()
        .input("tests/conv1d/conv1d.onnx")
        .out_dir("model/pretty_json/")
        .record_type(RecordType::PrettyJson)
        .run_from_script();

    ModelGen::new()
        .input("tests/conv1d/conv1d.onnx")
        .out_dir("model/pretty_json_half/")
        .record_type(RecordType::PrettyJson)
        .half_precision(true)
        .run_from_script();

    ModelGen::new()
        .input("tests/conv1d/conv1d.onnx")
        .out_dir("model/named_mpk_gz/")
        .record_type(RecordType::NamedMpkGz)
        .run_from_script();

    ModelGen::new()
        .input("tests/conv1d/conv1d.onnx")
        .out_dir("model/named_mpk_gz_half/")
        .record_type(RecordType::NamedMpkGz)
        .half_precision(true)
        .run_from_script();

    ModelGen::new()
        .input("tests/conv1d/conv1d.onnx")
        .out_dir("model/bincode/")
        .record_type(RecordType::Bincode)
        .run_from_script();

    ModelGen::new()
        .input("tests/conv1d/conv1d.onnx")
        .out_dir("model/bincode_half/")
        .record_type(RecordType::Bincode)
        .half_precision(true)
        .run_from_script();

    ModelGen::new()
        .input("tests/conv1d/conv1d.onnx")
        .out_dir("model/bincode_embedded/")
        .embed_states(true)
        .record_type(RecordType::Bincode)
        .run_from_script();

    ModelGen::new()
        .input("tests/conv1d/conv1d.onnx")
        .out_dir("model/bincode_embedded_half/")
        .embed_states(true)
        .half_precision(true)
        .record_type(RecordType::Bincode)
        .run_from_script();

    // panic!("Purposefully failing build to output logs.");
}
