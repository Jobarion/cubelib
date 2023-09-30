# Cubelib
Cubelib is a project that aims to provide a robust and feature rich and most importantly _fast_ library for dealing with 3x3 Rubik's cubes.

### Goals
- Provide simple APIs for manipulating Rubik's cubes (turning, rotating, inverting).
- Implement a solver that can efficiently get Rubik's cubes from one subgroup into another.
- Implement common subgroups that are useful for FMC solving for the solver mentioned above.
- Provide a simple-to-use API for defining custom subgroups.
- Offer a feature-rich, yet reasonably easy to understand CLI for FMC solving.
- Implement all of the above in a way that is fast while trying to minimize the use of pre-generated data that has to be stored on disk.

## Usage
Cubelib aims to define reasonable defaults to make using it as easy as possible.
To solve a scrambled cube, simply run `cubelib <scramble>`. This will return a single solution, broken down into steps. 

Because direct, optimal solutions are rarely ever useful, **Cubelib will by default first solve EO, then DR, then HTR, FR and finally the full cube**. This is the major difference between Cubelib and other tools.

Example of a simple solve:
```
> cubelib "R U R' D' L2 F2 B' R F B2 L2 B' D U2 F U D' R"

F' B' U2 R'             //eolr      (4/4)
U L2 F' U2 F D F2 L2 F  //drud-eolr (9/13)
B2 U' D F2 U            //htr-drud  (5/18)
D2                      //fr-ud     (1/19)
R2 B2                   //finish    (2/21)

Solution (21): F' B' U2 R' U L2 F' U2 F D F2 L2 F B2 U' D F2 U D2 R2 B2
```

There are a couple of options that can be set globally for all steps.

| Argument      | Description | 
| ------------ | ----------- |  
| `-N` `--niss` | Enables the usage of NISS. This applies differently to certain steps. Some steps like EO or HTR support using NISS anywhere during the step, for others like DR only a solution fully on the inverse will be searched by default |
| `-n <count>`  | By default only a single solution is returned. Use this option to define the number of solutions that will be returned |
| `-m <min>` `--min <min>`   | Return only solutions with a minimum length |
| `-M <max>` `--max <max>`   | Return only solutions with a maximum length |
| `-c` `--compact`   | Don't show intermediate steps, only print the final solution |
| `-p` `--plain`   | Don't print the length of each solution |
| `-a` `--all`   |  Print solutions that would otherwise get filtered out. E.g. an EO ending in F' |
| `-q <number>` `--quality <number>`   | Higher values result in better/shorter solutions, but they take longer to find. Use at your own risk. The default is `100` |
| `-l <number>` `--limit <number>`   | Similar to `--quality`. This property controlls how many solutions are carried from one step to the next. (e.g. on how many EOs we will attempt to find DRs). The default unlimited |
| `--optimal`   | Only find optimal solutions. This can take a long time |
| `-v` `--verbose`   | Print additional information for information and debugging purposes |
| `-q` `--quiet`   | Only print solutions and nothing else |
| `-s` `--steps`   | Configure the solver pipeline in detail. More information below |

## Step configurations
By default Cubelib always solves EO, then DR, HTR and FR before finishing the solve. This order, and the behaviour of these stages can be changed by providing a custom step configuration.
A step configuration is a list of steps separated by `>`. The default step configuration is `EO > DR > HTR > FR > FIN`. Each step can be configured by providing additional options. For example, to only look for EOs on UD or FB with a length of 3 to 5 moves, we could write `EO[ud;fb;min=3;max=5]`.

### Properties supported by all steps
| Option       | Description |
| ----       | ---- | 
| `min` | Minimum number of turns for this step, inclusive |  
| `max` | Maximum number of turns for this step, inclusive |
| `niss` | One of `none` (no NISS in this step), `before` (switching before the step is allowed), and `always` (switching during the step is allowed) |
| `quality` | Quality option for this step. See global options above for more information. |
| `limit` | Limit option for this step. See global options above for more information. |

If any of these properties are set, they will override global arguments. Any property not set will default to the global setting, or to (step specific) defaults.

### Steps
By default all steps are executed in all possible orientations. This can be changed for each step.

#### EO
Orients the edges on at least one axis. 

- Variations: `ud`, `fb`, `lr`.
- Prerequisite: -
- Default NISS option: `always`

#### RZP
Performs random moves
- Variations: -
- Prerequisite: EO
- Default NISS option: `before`

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

#### FR
Performs floppy reduction. This reduces the cube into a state where half moves on only two axis are sufficient.
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
- Prerequisite: FR or FRLS
- Default NISS option: `none`

### Examples
Find all EOs on the `ud` and `fb` axis between 2 and 5 moves, optionally using niss, and then to turn at most 10 of those EOs into DRs on the `fb` or `lr` axis without using NISS:
```
cubelib --steps "EO[ud;fb;min=2;max=5;niss=during;limit=10] > DR[fb;lr;niss=none]" <scramble>
```

To solve a cube into HTR using NISS anywhere except during DR:

```
cubelib --steps "EO[niss=always] > DR[niss=before] > HTR[niss=always]" <scramble>
```

Solve the cube, but always pick only the shortest solution for each step.
```
cubelib -l 1 <scramble>
```

### APIs
There is no API documentation yet. If you're interested in actually using this project please give me a shout be creating an issue.

## Future work
 - Support directly finishing from HTR or solving DRs without first doing EO.
 - Save pruning tables locally to start more quickly. This is very easy, but generating the current tables only takes a few seconds on modern hardware so this isn't a priority.
 - Support WebAssembly as a target and create JavaScript bindings.
 - Depending on WebAssembly support, deploy this project as a website.
 - Stabilize the API

## Acknowledgements
 - The tool [Nissy](https://nissy.tronto.net/) developed by Sebastiano Tronto served as an inspiration for this project. While the goals of Cubelib are different than Nissy's, there is also considerable overlap between the two. 
 - This project would not be possible without methods developed by Herbert Kociemba for [Cube Explorer](http://kociemba.org/cube.htm).
 - Morwen Thistlethwaite's Algorithm for solving the Rubik's cube is what made modern FMC solving possible. This project is essentially an implementation of a variant of this algorithm.
 - Jaap Scherphuis's description of orbit twists for the transition from Thistlethwaite's G2 to G3 are the only ones I could find on this fairly obscure topic, and were immensely helpful.