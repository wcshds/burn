#[burn_tensor_testgen::testgen(ad_conv_transpose2d)]
mod tests {
    use super::*;
    use burn_tensor::{module::conv_transpose2d, ops::ConvTransposeOptions, Data, Shape};

    #[test]
    fn test_conv_transpose2d_basic() {
        let test = ConvTranspose2dTestCase {
            batch_size: 2,
            channels: [2, 2],
            kernel_size: [3, 3],
            padding: [0, 0],
            padding_out: [0, 0],
            stride: [1, 1],
            dilation: [1, 1],
            groups: 1,
            size: [4, 4],
        };
        let device = Default::default();
        let grads = Grads {
            x: TestTensor::from_floats(
                [
                    [
                        [
                            [153., 153., 153., 153.],
                            [153., 153., 153., 153.],
                            [153., 153., 153., 153.],
                            [153., 153., 153., 153.],
                        ],
                        [
                            [477., 477., 477., 477.],
                            [477., 477., 477., 477.],
                            [477., 477., 477., 477.],
                            [477., 477., 477., 477.],
                        ],
                    ],
                    [
                        [
                            [153., 153., 153., 153.],
                            [153., 153., 153., 153.],
                            [153., 153., 153., 153.],
                            [153., 153., 153., 153.],
                        ],
                        [
                            [477., 477., 477., 477.],
                            [477., 477., 477., 477.],
                            [477., 477., 477., 477.],
                            [477., 477., 477., 477.],
                        ],
                    ],
                ],
                &device,
            ),
            weight: TestTensor::from_floats(
                [
                    [
                        [[752., 752., 752.], [752., 752., 752.], [752., 752., 752.]],
                        [[752., 752., 752.], [752., 752., 752.], [752., 752., 752.]],
                    ],
                    [
                        [
                            [1264., 1264., 1264.],
                            [1264., 1264., 1264.],
                            [1264., 1264., 1264.],
                        ],
                        [
                            [1264., 1264., 1264.],
                            [1264., 1264., 1264.],
                            [1264., 1264., 1264.],
                        ],
                    ],
                ],
                &device,
            ),
            bias: TestTensor::from_floats([72., 72.], &device),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv_transpose2d_padding() {
        let test = ConvTranspose2dTestCase {
            batch_size: 1,
            channels: [1, 1],
            kernel_size: [3, 3],
            padding: [1, 2],
            padding_out: [0, 0],
            stride: [1, 1],
            dilation: [1, 1],
            groups: 1,
            size: [4, 4],
        };
        let device = Default::default();
        let grads = Grads {
            x: TestTensor::from_floats(
                [[[
                    [13., 24., 20., 9.],
                    [15., 27., 21., 9.],
                    [15., 27., 21., 9.],
                    [7., 12., 8., 3.],
                ]]],
                &device,
            ),
            weight: TestTensor::from_floats(
                [[[[63., 57., 51.], [68., 60., 52.], [39., 33., 27.]]]],
                &device,
            ),
            bias: TestTensor::from_floats([8.], &device),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv_transpose2d_stride() {
        let test = ConvTranspose2dTestCase {
            batch_size: 1,
            channels: [1, 1],
            kernel_size: [3, 3],
            padding: [0, 0],
            padding_out: [0, 0],
            stride: [2, 3],
            dilation: [1, 1],
            groups: 1,
            size: [4, 4],
        };
        let device = Default::default();
        let grads = Grads {
            x: TestTensor::from_floats(
                [[[
                    [36., 36., 36., 36.],
                    [36., 36., 36., 36.],
                    [36., 36., 36., 36.],
                    [36., 36., 36., 36.],
                ]]],
                &device,
            ),
            weight: TestTensor::from_floats(
                [[[[120., 120., 120.], [120., 120., 120.], [120., 120., 120.]]]],
                &device,
            ),
            bias: TestTensor::from_floats([108.], &device),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv_transpose2d_stride_padding_out() {
        let test = ConvTranspose2dTestCase {
            batch_size: 1,
            channels: [1, 1],
            kernel_size: [3, 3],
            padding: [0, 0],
            padding_out: [1, 2],
            stride: [2, 3],
            dilation: [1, 1],
            groups: 1,
            size: [4, 4],
        };
        let device = Default::default();
        let grads = Grads {
            x: TestTensor::from_floats(
                [[[
                    [36., 36., 36., 36.],
                    [36., 36., 36., 36.],
                    [36., 36., 36., 36.],
                    [36., 36., 36., 36.],
                ]]],
                &device,
            ),
            weight: TestTensor::from_floats(
                [[[[120., 120., 120.], [120., 120., 120.], [120., 120., 120.]]]],
                &device,
            ),
            bias: TestTensor::from_floats([140.], &device),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv_transpose2d_dilation() {
        let test = ConvTranspose2dTestCase {
            batch_size: 1,
            channels: [1, 1],
            kernel_size: [3, 3],
            padding: [0, 0],
            padding_out: [0, 0],
            stride: [1, 1],
            dilation: [2, 3],
            groups: 1,
            size: [4, 4],
        };
        let device = Default::default();
        let grads = Grads {
            x: TestTensor::from_floats(
                [[[
                    [36., 36., 36., 36.],
                    [36., 36., 36., 36.],
                    [36., 36., 36., 36.],
                    [36., 36., 36., 36.],
                ]]],
                &device,
            ),
            weight: TestTensor::from_floats(
                [[[[120., 120., 120.], [120., 120., 120.], [120., 120., 120.]]]],
                &device,
            ),
            bias: TestTensor::from_floats([80.], &device),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv_transpose2d_channels() {
        let test = ConvTranspose2dTestCase {
            batch_size: 1,
            channels: [2, 3],
            kernel_size: [3, 3],
            padding: [0, 0],
            padding_out: [0, 0],
            stride: [1, 1],
            dilation: [1, 1],
            groups: 1,
            size: [4, 4],
        };
        let device = Default::default();
        let grads = Grads {
            x: TestTensor::from_floats(
                [[
                    [
                        [351., 351., 351., 351.],
                        [351., 351., 351., 351.],
                        [351., 351., 351., 351.],
                        [351., 351., 351., 351.],
                    ],
                    [
                        [1080., 1080., 1080., 1080.],
                        [1080., 1080., 1080., 1080.],
                        [1080., 1080., 1080., 1080.],
                        [1080., 1080., 1080., 1080.],
                    ],
                ]],
                &device,
            ),
            weight: TestTensor::from_floats(
                [
                    [
                        [[120., 120., 120.], [120., 120., 120.], [120., 120., 120.]],
                        [[120., 120., 120.], [120., 120., 120.], [120., 120., 120.]],
                        [[120., 120., 120.], [120., 120., 120.], [120., 120., 120.]],
                    ],
                    [
                        [[376., 376., 376.], [376., 376., 376.], [376., 376., 376.]],
                        [[376., 376., 376.], [376., 376., 376.], [376., 376., 376.]],
                        [[376., 376., 376.], [376., 376., 376.], [376., 376., 376.]],
                    ],
                ],
                &device,
            ),
            bias: TestTensor::from_floats([36., 36., 36.], &device),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv_transpose2d_kernel_size() {
        let test = ConvTranspose2dTestCase {
            batch_size: 1,
            channels: [1, 1],
            kernel_size: [3, 5],
            padding: [0, 0],
            padding_out: [0, 0],
            stride: [1, 1],
            dilation: [1, 1],
            groups: 1,
            size: [6, 6],
        };
        let device = Default::default();
        let grads = Grads {
            x: TestTensor::from_floats(
                [[[
                    [105., 105., 105., 105., 105., 105.],
                    [105., 105., 105., 105., 105., 105.],
                    [105., 105., 105., 105., 105., 105.],
                    [105., 105., 105., 105., 105., 105.],
                    [105., 105., 105., 105., 105., 105.],
                    [105., 105., 105., 105., 105., 105.],
                ]]],
                &device,
            ),
            weight: TestTensor::from_floats(
                [[[
                    [630., 630., 630., 630., 630.],
                    [630., 630., 630., 630., 630.],
                    [630., 630., 630., 630., 630.],
                ]]],
                &device,
            ),
            bias: TestTensor::from_floats([80.], &device),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv_transpose2d_groups() {
        let test = ConvTranspose2dTestCase {
            batch_size: 1,
            channels: [2, 2],
            kernel_size: [3, 3],
            padding: [0, 0],
            padding_out: [0, 0],
            stride: [1, 1],
            dilation: [1, 1],
            groups: 2,
            size: [4, 4],
        };
        let device = Default::default();
        let grads = Grads {
            x: TestTensor::from_floats(
                [[
                    [
                        [36., 36., 36., 36.],
                        [36., 36., 36., 36.],
                        [36., 36., 36., 36.],
                        [36., 36., 36., 36.],
                    ],
                    [
                        [117., 117., 117., 117.],
                        [117., 117., 117., 117.],
                        [117., 117., 117., 117.],
                        [117., 117., 117., 117.],
                    ],
                ]],
                &device,
            ),
            weight: TestTensor::from_floats(
                [
                    [[[120., 120., 120.], [120., 120., 120.], [120., 120., 120.]]],
                    [[[376., 376., 376.], [376., 376., 376.], [376., 376., 376.]]],
                ],
                &device,
            ),
            bias: TestTensor::from_floats([36., 36.], &device),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv_transpose2d_complex_no_groups() {
        let test = ConvTranspose2dTestCase {
            batch_size: 2,
            channels: [2, 3],
            kernel_size: [3, 5],
            padding: [1, 2],
            padding_out: [1, 2],
            stride: [2, 3],
            dilation: [2, 3],
            groups: 1,
            size: [6, 8],
        };
        let device = Default::default();
        let grads = Grads {
            x: TestTensor::from_floats(
                [
                    [
                        [
                            [600., 735., 735., 735., 735., 735., 735., 735.],
                            [810., 990., 990., 990., 990., 990., 990., 990.],
                            [810., 990., 990., 990., 990., 990., 990., 990.],
                            [810., 990., 990., 990., 990., 990., 990., 990.],
                            [810., 990., 990., 990., 990., 990., 990., 990.],
                            [810., 990., 990., 990., 990., 990., 990., 990.],
                        ],
                        [
                            [1680., 2085., 2085., 2085., 2085., 2085., 2085., 2085.],
                            [2430., 3015., 3015., 3015., 3015., 3015., 3015., 3015.],
                            [2430., 3015., 3015., 3015., 3015., 3015., 3015., 3015.],
                            [2430., 3015., 3015., 3015., 3015., 3015., 3015., 3015.],
                            [2430., 3015., 3015., 3015., 3015., 3015., 3015., 3015.],
                            [2430., 3015., 3015., 3015., 3015., 3015., 3015., 3015.],
                        ],
                    ],
                    [
                        [
                            [600., 735., 735., 735., 735., 735., 735., 735.],
                            [810., 990., 990., 990., 990., 990., 990., 990.],
                            [810., 990., 990., 990., 990., 990., 990., 990.],
                            [810., 990., 990., 990., 990., 990., 990., 990.],
                            [810., 990., 990., 990., 990., 990., 990., 990.],
                            [810., 990., 990., 990., 990., 990., 990., 990.],
                        ],
                        [
                            [1680., 2085., 2085., 2085., 2085., 2085., 2085., 2085.],
                            [2430., 3015., 3015., 3015., 3015., 3015., 3015., 3015.],
                            [2430., 3015., 3015., 3015., 3015., 3015., 3015., 3015.],
                            [2430., 3015., 3015., 3015., 3015., 3015., 3015., 3015.],
                            [2430., 3015., 3015., 3015., 3015., 3015., 3015., 3015.],
                            [2430., 3015., 3015., 3015., 3015., 3015., 3015., 3015.],
                        ],
                    ],
                ],
                &device,
            ),
            weight: TestTensor::from_floats(
                [
                    [
                        [
                            [5320., 6040., 6040., 6040., 6040.],
                            [6048., 6864., 6864., 6864., 6864.],
                            [6048., 6864., 6864., 6864., 6864.],
                        ],
                        [
                            [5320., 6040., 6040., 6040., 6040.],
                            [6048., 6864., 6864., 6864., 6864.],
                            [6048., 6864., 6864., 6864., 6864.],
                        ],
                        [
                            [5320., 6040., 6040., 6040., 6040.],
                            [6048., 6864., 6864., 6864., 6864.],
                            [6048., 6864., 6864., 6864., 6864.],
                        ],
                    ],
                    [
                        [
                            [8680., 9880., 9880., 9880., 9880.],
                            [10080., 11472., 11472., 11472., 11472.],
                            [10080., 11472., 11472., 11472., 11472.],
                        ],
                        [
                            [8680., 9880., 9880., 9880., 9880.],
                            [10080., 11472., 11472., 11472., 11472.],
                            [10080., 11472., 11472., 11472., 11472.],
                        ],
                        [
                            [8680., 9880., 9880., 9880., 9880.],
                            [10080., 11472., 11472., 11472., 11472.],
                            [10080., 11472., 11472., 11472., 11472.],
                        ],
                    ],
                ],
                &device,
            ),
            bias: TestTensor::from_floats([896., 896., 896.], &device),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv_transpose2d_complex_no_groups_2() {
        let test = ConvTranspose2dTestCase {
            batch_size: 1,
            channels: [4, 2],
            kernel_size: [2, 3],
            padding: [1, 2],
            padding_out: [1, 2],
            stride: [2, 3],
            dilation: [1, 2],
            groups: 1,
            size: [10, 10],
        };
        let device = Default::default();
        let grads = Grads {
            x: TestTensor::from_floats(
                [[
                    [
                        [30., 42., 42., 42., 42., 42., 42., 42., 42., 42.],
                        [48., 66., 66., 66., 66., 66., 66., 66., 66., 66.],
                        [48., 66., 66., 66., 66., 66., 66., 66., 66., 66.],
                        [48., 66., 66., 66., 66., 66., 66., 66., 66., 66.],
                        [48., 66., 66., 66., 66., 66., 66., 66., 66., 66.],
                        [48., 66., 66., 66., 66., 66., 66., 66., 66., 66.],
                        [48., 66., 66., 66., 66., 66., 66., 66., 66., 66.],
                        [48., 66., 66., 66., 66., 66., 66., 66., 66., 66.],
                        [48., 66., 66., 66., 66., 66., 66., 66., 66., 66.],
                        [48., 66., 66., 66., 66., 66., 66., 66., 66., 66.],
                    ],
                    [
                        [78., 114., 114., 114., 114., 114., 114., 114., 114., 114.],
                        [144., 210., 210., 210., 210., 210., 210., 210., 210., 210.],
                        [144., 210., 210., 210., 210., 210., 210., 210., 210., 210.],
                        [144., 210., 210., 210., 210., 210., 210., 210., 210., 210.],
                        [144., 210., 210., 210., 210., 210., 210., 210., 210., 210.],
                        [144., 210., 210., 210., 210., 210., 210., 210., 210., 210.],
                        [144., 210., 210., 210., 210., 210., 210., 210., 210., 210.],
                        [144., 210., 210., 210., 210., 210., 210., 210., 210., 210.],
                        [144., 210., 210., 210., 210., 210., 210., 210., 210., 210.],
                        [144., 210., 210., 210., 210., 210., 210., 210., 210., 210.],
                    ],
                    [
                        [126., 186., 186., 186., 186., 186., 186., 186., 186., 186.],
                        [240., 354., 354., 354., 354., 354., 354., 354., 354., 354.],
                        [240., 354., 354., 354., 354., 354., 354., 354., 354., 354.],
                        [240., 354., 354., 354., 354., 354., 354., 354., 354., 354.],
                        [240., 354., 354., 354., 354., 354., 354., 354., 354., 354.],
                        [240., 354., 354., 354., 354., 354., 354., 354., 354., 354.],
                        [240., 354., 354., 354., 354., 354., 354., 354., 354., 354.],
                        [240., 354., 354., 354., 354., 354., 354., 354., 354., 354.],
                        [240., 354., 354., 354., 354., 354., 354., 354., 354., 354.],
                        [240., 354., 354., 354., 354., 354., 354., 354., 354., 354.],
                    ],
                    [
                        [174., 258., 258., 258., 258., 258., 258., 258., 258., 258.],
                        [336., 498., 498., 498., 498., 498., 498., 498., 498., 498.],
                        [336., 498., 498., 498., 498., 498., 498., 498., 498., 498.],
                        [336., 498., 498., 498., 498., 498., 498., 498., 498., 498.],
                        [336., 498., 498., 498., 498., 498., 498., 498., 498., 498.],
                        [336., 498., 498., 498., 498., 498., 498., 498., 498., 498.],
                        [336., 498., 498., 498., 498., 498., 498., 498., 498., 498.],
                        [336., 498., 498., 498., 498., 498., 498., 498., 498., 498.],
                        [336., 498., 498., 498., 498., 498., 498., 498., 498., 498.],
                        [336., 498., 498., 498., 498., 498., 498., 498., 498., 498.],
                    ],
                ]],
                &device,
            ),
            weight: TestTensor::from_floats(
                [
                    [
                        [[4455., 4905., 4905.], [4500., 4950., 4950.]],
                        [[4455., 4905., 4905.], [4500., 4950., 4950.]],
                    ],
                    [
                        [[12555., 13905., 13905.], [13500., 14950., 14950.]],
                        [[12555., 13905., 13905.], [13500., 14950., 14950.]],
                    ],
                    [
                        [[20655., 22905., 22905.], [22500., 24950., 24950.]],
                        [[20655., 22905., 22905.], [22500., 24950., 24950.]],
                    ],
                    [
                        [[28755., 31905., 31905.], [31500., 34950., 34950.]],
                        [[28755., 31905., 31905.], [31500., 34950., 34950.]],
                    ],
                ],
                &device,
            ),
            bias: TestTensor::from_floats([570., 570.], &device),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv_transpose2d_complex_groups() {
        let test = ConvTranspose2dTestCase {
            batch_size: 1,
            channels: [4, 2],
            kernel_size: [2, 3],
            padding: [1, 2],
            padding_out: [1, 2],
            stride: [2, 3],
            dilation: [1, 2],
            groups: 2,
            size: [10, 10],
        };
        let device = Default::default();
        let grads = Grads {
            x: TestTensor::from_floats(
                [[
                    [
                        [9., 12., 12., 12., 12., 12., 12., 12., 12., 12.],
                        [12., 15., 15., 15., 15., 15., 15., 15., 15., 15.],
                        [12., 15., 15., 15., 15., 15., 15., 15., 15., 15.],
                        [12., 15., 15., 15., 15., 15., 15., 15., 15., 15.],
                        [12., 15., 15., 15., 15., 15., 15., 15., 15., 15.],
                        [12., 15., 15., 15., 15., 15., 15., 15., 15., 15.],
                        [12., 15., 15., 15., 15., 15., 15., 15., 15., 15.],
                        [12., 15., 15., 15., 15., 15., 15., 15., 15., 15.],
                        [12., 15., 15., 15., 15., 15., 15., 15., 15., 15.],
                        [12., 15., 15., 15., 15., 15., 15., 15., 15., 15.],
                    ],
                    [
                        [21., 30., 30., 30., 30., 30., 30., 30., 30., 30.],
                        [36., 51., 51., 51., 51., 51., 51., 51., 51., 51.],
                        [36., 51., 51., 51., 51., 51., 51., 51., 51., 51.],
                        [36., 51., 51., 51., 51., 51., 51., 51., 51., 51.],
                        [36., 51., 51., 51., 51., 51., 51., 51., 51., 51.],
                        [36., 51., 51., 51., 51., 51., 51., 51., 51., 51.],
                        [36., 51., 51., 51., 51., 51., 51., 51., 51., 51.],
                        [36., 51., 51., 51., 51., 51., 51., 51., 51., 51.],
                        [36., 51., 51., 51., 51., 51., 51., 51., 51., 51.],
                        [36., 51., 51., 51., 51., 51., 51., 51., 51., 51.],
                    ],
                    [
                        [33., 48., 48., 48., 48., 48., 48., 48., 48., 48.],
                        [60., 87., 87., 87., 87., 87., 87., 87., 87., 87.],
                        [60., 87., 87., 87., 87., 87., 87., 87., 87., 87.],
                        [60., 87., 87., 87., 87., 87., 87., 87., 87., 87.],
                        [60., 87., 87., 87., 87., 87., 87., 87., 87., 87.],
                        [60., 87., 87., 87., 87., 87., 87., 87., 87., 87.],
                        [60., 87., 87., 87., 87., 87., 87., 87., 87., 87.],
                        [60., 87., 87., 87., 87., 87., 87., 87., 87., 87.],
                        [60., 87., 87., 87., 87., 87., 87., 87., 87., 87.],
                        [60., 87., 87., 87., 87., 87., 87., 87., 87., 87.],
                    ],
                    [
                        [45., 66., 66., 66., 66., 66., 66., 66., 66., 66.],
                        [84., 123., 123., 123., 123., 123., 123., 123., 123., 123.],
                        [84., 123., 123., 123., 123., 123., 123., 123., 123., 123.],
                        [84., 123., 123., 123., 123., 123., 123., 123., 123., 123.],
                        [84., 123., 123., 123., 123., 123., 123., 123., 123., 123.],
                        [84., 123., 123., 123., 123., 123., 123., 123., 123., 123.],
                        [84., 123., 123., 123., 123., 123., 123., 123., 123., 123.],
                        [84., 123., 123., 123., 123., 123., 123., 123., 123., 123.],
                        [84., 123., 123., 123., 123., 123., 123., 123., 123., 123.],
                        [84., 123., 123., 123., 123., 123., 123., 123., 123., 123.],
                    ],
                ]],
                &device,
            ),
            weight: TestTensor::from_floats(
                [
                    [[[4455., 4905., 4905.], [4500., 4950., 4950.]]],
                    [[[12555., 13905., 13905.], [13500., 14950., 14950.]]],
                    [[[20655., 22905., 22905.], [22500., 24950., 24950.]]],
                    [[[28755., 31905., 31905.], [31500., 34950., 34950.]]],
                ],
                &device,
            ),
            bias: TestTensor::from_floats([570., 570.], &device),
        };
        test.assert_grads(grads);
    }

    struct ConvTranspose2dTestCase {
        batch_size: usize,
        channels: [usize; 2],
        kernel_size: [usize; 2],
        padding: [usize; 2],
        padding_out: [usize; 2],
        stride: [usize; 2],
        dilation: [usize; 2],
        groups: usize,
        size: [usize; 2],
    }

    struct Grads {
        x: TestTensor<4>,
        weight: TestTensor<4>,
        bias: TestTensor<1>,
    }

    impl ConvTranspose2dTestCase {
        fn assert_grads(self, expected_grads: Grads) {
            let shape_x = Shape::new([
                self.batch_size,
                self.channels[0],
                self.size[0],
                self.size[1],
            ]);
            let shape_weight = Shape::new([
                self.channels[0],
                self.channels[1] / self.groups,
                self.kernel_size[0],
                self.kernel_size[1],
            ]);
            let device = Default::default();
            let weight = TestAutodiffTensor::from_data(
                TestTensorInt::arange(0..shape_weight.num_elements() as i64, &device)
                    .reshape(shape_weight)
                    .into_data()
                    .convert(),
                &device,
            )
            .require_grad();
            let bias = TestAutodiffTensor::from_data(
                TestTensorInt::arange(0..self.channels[1] as i64, &device)
                    .into_data()
                    .convert(),
                &device,
            )
            .require_grad();
            let x = TestAutodiffTensor::from_data(
                TestTensorInt::arange(0..shape_x.num_elements() as i64, &device)
                    .reshape(shape_x)
                    .into_data()
                    .convert(),
                &device,
            )
            .require_grad();
            let output = conv_transpose2d(
                x.clone(),
                weight.clone(),
                Some(bias.clone()),
                ConvTransposeOptions::new(
                    self.stride,
                    self.padding,
                    self.padding_out,
                    self.dilation,
                    self.groups,
                ),
            );
            let grads = output.backward();

            // Assert
            let x_grad_actual = x.grad(&grads).unwrap();
            let weight_grad_actual = weight.grad(&grads).unwrap();
            let bias_grad_actual = bias.grad(&grads).unwrap();

            expected_grads
                .bias
                .to_data()
                .assert_approx_eq(&bias_grad_actual.to_data(), 3);
            expected_grads
                .x
                .to_data()
                .assert_approx_eq(&x_grad_actual.to_data(), 3);
            expected_grads
                .weight
                .to_data()
                .assert_approx_eq(&weight_grad_actual.to_data(), 3);
        }
    }
}
