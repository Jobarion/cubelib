use crate::cubie::CubieCube;

trait CornerOrientationCountUD {
    fn co_count(&self) -> u8;
}

trait CornerOrientationCountFB {
    fn co_count(&self) -> u8;
}

trait CornerOrientationCountRL {
    fn co_count(&self) -> u8;
}

trait CornerOrientationCount {
    fn co_count_all(&self) -> (u8, u8, u8);
}

impl <CO: CornerOrientationCountUD + CornerOrientationCountRL + CornerOrientationCountFB> CornerOrientationCount for CO {
    fn co_count_all(&self) -> (u8, u8, u8) {
        (
            (CornerOrientationCountUD::co_count(self)),
            (CornerOrientationCountFB::co_count(self)),
            (CornerOrientationCountRL::co_count(self))
        )
    }
}

impl CornerOrientationCountUD for CubieCube {
    fn co_count(&self) -> u8 {
        todo!()
    }
}