use std::str::FromStr;

use log::LevelFilter;
use simple_logger::SimpleLogger;

use cubelib::algs::Algorithm;
use cubelib::cube::Cube333;
use cubelib::cube::turn::TurnableMut;
use cubelib::defs::NissSwitchType;
use cubelib::solver::lookup_table::LookupTable;
use cubelib::solver_new::dr::DRStep;
use cubelib::solver_new::eo::EOStep;
use cubelib::solver_new::group::StepGroup;
use cubelib::steps::dr::coords::ARMUDToDRCoord;

pub type ARMPruningTable = LookupTable<{ 4900 }, ARMUDToDRCoord>;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .init();

    let mut cube: Cube333 = Algorithm::from_str("R' U' F D2 B F U2 F L R' B L B D' B' R2 B' L R2 F' R' U' F").unwrap().into();

    let eo_step = EOStep::builder()
        .max_length(5)
        .niss(NissSwitchType::Never)
        .build();

    let dr_step = DRStep::builder()
        .max_length(6)
        // .max_absolute_length(11)
        .niss(NissSwitchType::Before)
        .subsets(cubelib::steps::util::SUBSETS_4A1.into())
        .build();

    let par = StepGroup::sequential(vec![eo_step, dr_step]);
    let mut worker = par.into_worker(cube);

    println!("{:?}", worker.next());



    //
    // println!("{:?}", ARMDRCOCoord::from(&cube.corners));
    // println!("{:?}", ARMDREdgesCoord::from(&cube.edges));
    // println!("{:?}", cube.corners.0);
    // println!("{:?}", cube.is_arm());
    // println!("{cube}");
    //
    // // let table = lookup_table::generate(&ARM_UD_EO_FB_MOVESET,
    // //                        &|c: &Cube333| {
    // //                            // println!("{:?}", ARMCOCoord::from(&cube.corners));
    // //                            // println!("{:?}", ARMEdgesCoord::from(&cube.edges));
    // //                            // println!("{:?}", ARMUDCoord::from(&cube));
    // //
    // //                            ARMUDCoord::from(c)
    // //                        },
    // //                        &|| ARMPruningTable::new(false),
    // //                        &|table, coord|table.get(coord),
    // //                        &|table, coord, val|table.set(coord, val));

}