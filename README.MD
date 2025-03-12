# Cubelib
Cubelib is a project that aims to provide a robust and feature rich and most importantly _fast_ library for dealing with 3x3 Rubik's cubes.
See https://joba.me/mallard for a website that's using cubelib to generate human-findable FMC solutions.

### Goals
- Provide simple APIs for manipulating Rubik's cubes (turning, rotating, inverting).
- Implement a solver that can efficiently get Rubik's cubes from one subgroup into another.
- Implement common subgroups that are useful for FMC solving for the solver mentioned above.
- Provide a simple-to-use API for defining custom subgroups.
- Offer a feature-rich, yet reasonably easy to understand CLI for FMC solving.
- Implement all of the above in a way that is fast while trying to minimize the use of pre-generated data that has to be stored on disk.

## CLI Solve

Cubelib aims to define reasonable defaults to make using it as easy as possible.
To solve a scrambled cube, simply run `cubelib <scramble>`. This will return a single solution, broken down into steps. 

Because direct, optimal solutions are rarely ever useful, **Cubelib will by default first solve EO, then DR, then HTR, FR and finally the full cube**. It will also try to pick reasonable defaults for each steps, including restricting DRs to commonly known triggers. This is the major difference between Cubelib and other tools.

Example of a simple solve:
```
> cubelib solve --niss --quality 5000 "R' U' F R2 U2 L2 F2 D2 R2 B' U2 L2 B F' R B' F' R' U2 B' R2 U' L' D R' U' F"

(B' D' L')            //eo-lr      (3/3)
(B D)                 //rzp-lr     (2/5)
F2 D F2 L2 D' F       //dr-ud-eolr (6/11)
L2 U2 B2 D'           //htr-ud     (4/15)
B2 R2 D2 B2 D2 F2 R2  //finish     (7/22)

Solution (22): F2 D F2 L2 D' F L2 U2 B2 D' B2 R2 D2 B2 D2 F2 R2 D' B' L D B
```

There are a couple of options that can be set globally for all steps.

| Argument                           | Description                                                                                                                                                                                                                       | 
|------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|  
| `-N` `--niss`                      | Enables the usage of NISS. This applies differently to certain steps. Some steps like EO or HTR support using NISS anywhere during the step, for others like DR only a solution fully on the inverse will be searched by default  |
| `-n <count>`                       | By default only a single solution is returned. Use this option to define the number of solutions that will be returned                                                                                                            |
| `-m <min>` `--min <min>`           | Return only solutions with a minimum length                                                                                                                                                                                       |
| `-M <max>` `--max <max>`           | Return only solutions with a maximum length                                                                                                                                                                                       |
| `-c` `--compact`                   | Don't show intermediate steps, only print the final solution                                                                                                                                                                      |
| `-p` `--plain`                     | Don't print the length of each solution                                                                                                                                                                                           |
| `-a` `--all`                       | Print solutions that would otherwise get filtered out. E.g. an EO ending in F'                                                                                                                                                    |
| `-q <number>` `--quality <number>` | Higher values result in better/shorter solutions, but they take longer to find. Set to `0` for optimal search. The default is `100`                                                                                               |
| `-v` `--verbose`                   | Print additional information for information and debugging purposes                                                                                                                                                               |
| `--quiet`                          | Only print solutions and nothing else                                                                                                                                                                                             |
| `-s` `--steps`                     | Configure the solver pipeline. More information below. Defaults to `EO > RZP > DR[triggers=R,RU2R,RU'R] > HTR > FIN`                                                                                                              | 

## Step configurations
By default Cubelib always solves EO, then DR, HTR and FR before finishing the solve. This order, and the behaviour of these stages can be changed by providing a custom step configuration.
A step configuration is a list of steps separated by `>`. The default step configuration is `EO > RZP > DR[triggers=R,RU2R,RU'R] > HTR > FIN`. Each step can be configured by providing additional options. For example, to only look for EOs on UD or FB with a length of 3 to 5 moves, we could write `EO[ud;fb;min=3;max=5]`.

### Properties supported by all steps
| Option    | Description                                                                                                                                                     |
|-----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------| 
| `min`     | Minimum number of turns for this step, inclusive                                                                                                                |  
| `max`     | Maximum number of turns for this step, inclusive                                                                                                                |
| `niss`    | One of `none` (keep the orientation of the previous step), `before` (switching before the step is allowed), and `always` (switching during the step is allowed) |
| `limit`   | Limit option for this step. See global options above for more information.                                                                                      |

If any of these properties are set, they will override global arguments. Any property not set will default to the global setting, or to (step specific) defaults.

### Steps
By default, all steps are executed in all possible orientations. This can be changed for each step.

#### EO
Orients the edges on at least one axis. 

- Variations: `ud`, `fb`, `lr`.
- Prerequisite: -
- Default NISS option: `always`

#### RZP
Performs random moves
- Variations: -
- Prerequisite: EO
- Default NISS option: `none`

#### DR
Orients the edges on a second axis, and orients the corners. 
 - Variations: 
   - `ud`, `fb`, `lr`, for DR on that axis with any EO axis. 
   - `drud-eofb`, `drud-eolr`, `drfb-eoud`, `drfb-eolr`, `drlr-eoud`, `drlr-eofb` to be specific about both the DR and EO axis.
 - Prerequisite: EO or RZP
 - Default NISS option: `before`
 - Additional options
   - DR can be restricted to specific triggers by setting `triggers=<trigger1>,<trigger2>,...`. 
     E.g. `triggers=RUR,RU2R,R` would allow only those three triggers **in all possible orientations**. (i.e. `L U' L` is allowed, `R U' R` wouldn't. Inverting the last move is also always allowed).
     This option implicitly adds an RZP step with default options if one wasn't already defined. If you would like to avoid that, define an RZP step manually with `max=0`.
     

#### HTR
Reduces the cube into a state that is solvable using only half turns. 
- Variations: `ud`, `fb`, `lr`.
- Prerequisite: DR
- Default NISS option: `before`
- Additional options
  - HTR can be restricted to certain subsets using the `subset` parameter with a comma separated list of subsets.
  - `subsets=2c3,4a1` restricts to these two specific HTR subsets
  - `subsets=1,2,2c3 4e` restricts to all 1 and 2 QT subsets + 2c3 with 4 bad edges.

#### FR
Performs floppy reduction. This reduces the cube into a state where half moves on only two axis are sufficient. This step can be skipped if a direct solution from HTR to solved is desired.
 - Variations: `ud`, `fb`, `lr`.
 - Prerequisite: HTR
- Default NISS option: `before`

#### FRLS
Performs floppy reduction while ignoring the slice edges. Using this step will later require using insertions. 
Cubelib does not yet support insertions, so these will have to be done manually.
- Variations: `ud`, `fb`, `lr`.
- Prerequisite: HTR
- Default NISS option: `before`

#### FIN
Solves the cube.
- Variations: -
- Prerequisite: FR, FRLS or HTR
- Default NISS option: `none`

### Examples
Find all EOs on the `ud` and `fb` axis between 2 and 5 moves, optionally using niss, and then to turn at most 10 of those EOs into DRs on the `fb` or `lr` axis without using NISS:
```
cubelib solve --steps "EO[ud;fb;min=2;max=5;niss=always;limit=10] > DR[fb;lr;niss=none]" <scramble>
```

To solve a cube into HTR using NISS anywhere except during DR:

```
cubelib solve --steps "EO[niss=always] > DR[niss=before] > HTR[niss=always]" <scramble>
```

Find the shortest DR that ends with the 4c2e trigger `R U2 R'` or the 4c4e trigger `R` (with default EO and RZP settings):
```
cubelib solve -q 0 --steps "EO > DR[triggers=RU2R,R]" <scramble>
```

## CLI Scramble
To generate a scramble:
```commandline
cubelib scramble
```

## CLI Invert
```commandline
cubelib invert <scramble>
```


## APIs
There is no API documentation yet. If you're interested in actually using this project please let me know by creating an issue.

## Future work
 - Support directly finishing from HTR or solving DRs without first doing EO.
 - Save pruning tables locally to start more quickly. This is very easy, but generating the current tables only takes a few seconds on modern hardware so this isn't a priority.
 - Support WebAssembly as a target and create JavaScript bindings.
 - Depending on WebAssembly support, deploy this project as a website.
 - Stabilize the API

## Acknowledgements
 - The tool [Nissy](https://nissy.tronto.net/) developed by Sebastiano Tronto served as an inspiration for this project. While the goals of Cubelib are different than Nissy's, there is also considerable overlap between the two. Consider checking it out especially if you're interested in finding direct optimal solutions for DR / HTR.
 - This project would not be possible without methods developed by Herbert Kociemba for the Two Step Algorithm and [Cube Explorer](http://kociemba.org/cube.htm).
 - Morwen Thistlethwaite's Algorithm for solving the Rubik's cube is what made modern FMC solving possible. This project is essentially an implementation of a variant of this algorithm.
 - Jaap Scherphuis's description of orbit twists for the transition from Thistlethwaite's G2 to G3 was the only one I could find on this fairly obscure topic, and it was immensely helpful.
