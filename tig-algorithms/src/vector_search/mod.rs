pub mod optimax_gpu;
pub use optimax_gpu as c004_a026;

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use std::time::Instant;
    use tig_challenges::{vector_search::*, *};
    use std::time::{SystemTime, UNIX_EPOCH};

    
    pub fn time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    fn random_difficulty() -> Difficulty {
        let mut rng = StdRng::seed_from_u64(time() as u64);
        let num_queries = rng.gen_range(100..=500); // Générer un nombre aléatoire de requêtes
        let better_than_baseline = rng.gen_range(501..=580); // Générer une distance baseline aléatoire

        Difficulty {
            num_queries,
            better_than_baseline,
        }
    }

    fn random_seed() -> [u64; 8] {
        let mut rng = StdRng::seed_from_u64(time() as u64);
        let mut seed = [0u64; 8];
        for i in 0..8 {
            seed[i] = rng.gen_range(1..=1_000_000); // Générer des seeds aléatoires
        }
        seed
    }

    #[test]
    fn test_compare_solve_challenges() {
        // Générer une difficulté aléatoire et une seed aléatoire
        let mut i = 0;

        while i < 300 {
            let difficulty = random_difficulty();
            let seed = random_seed();
            let challenge = Challenge::generate_instance(seed, &difficulty).unwrap();

            println!(
                "Running test with difficulty: num_queries = {}, better_than_baseline = {}",
                difficulty.num_queries, difficulty.better_than_baseline
            );

            // // Test de l'algorithme solve_challenge (actuel)
            let start_time = Instant::now();
            // let result_current = optimax_gpu::solve_challenge_old(&challenge);
            //let duration_current = start_time.elapsed();

            // match result_current {
            //     Ok(Some(solution)) => match challenge.verify_solution(&solution) {
            //         Ok(_) => println!("Valid solution (solve_challenge) ... ok"),
            //         Err(e) => println!("Invalid solution (solve_challenge): {}", e),
            //     },
            //     Ok(None) => println!("No solution (solve_challenge)"),
            //     Err(e) => println!("Algorithm error (solve_challenge): {}", e),
            // };

            // Test de l'algorithme solve_challenge_test (nouveau)
            let start_time_test = Instant::now();
            let result_test = optimax_gpu::solve_challenge(&challenge);
            let duration_test = start_time_test.elapsed();

            match result_test {
                Ok(Some(solution)) => match challenge.verify_solution(&solution) {
                    Ok(_) => println!("Valid solution (solve_challenge_test) ... ok"),
                    Err(e) => {}, //println!("KO Invalid solution (solve_challenge_test): {}", e),
                },
                Ok(None) => println!("No solution (solve_challenge_test)"),
                Err(e) => println!("Algorithm error (solve_challenge_test): {}", e),
            };

            // Afficher les résultats de la comparaison de performance
            // println!("Performance comparison:");
            // println!("solve_challenge (current) duration: {:?}", duration_current);
            // println!("solve_challenge_test (new) duration: {:?}", duration_test);
            i += 1;
        }
    }
}

#[cfg(feature = "cuda")]
#[cfg(test)]

mod cuda_tests {
    use std::collections::HashMap;

    use super::*;
    use cudarc::driver::*;
    use cudarc::nvrtc::compile_ptx;
    use std::sync::Arc;
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use tig_challenges::{vector_search::*, *};
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::time::Instant;
    
    pub fn time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    fn random_difficulty() -> Difficulty {
        let mut rng = StdRng::seed_from_u64(time() as u64);
        let num_queries = rng.gen_range(100..=117); // Générer un nombre aléatoire de requêtes
        let better_than_baseline = rng.gen_range(430..=450); // Générer une distance baseline aléatoire

        Difficulty {
            num_queries,
            better_than_baseline,
        }
    }

    fn random_seed() -> [u64; 8] {
        let mut rng = StdRng::seed_from_u64(time() as u64);
        let mut seed = [0u64; 8];
        for i in 0..8 {
            seed[i] = rng.gen_range(1..=1_000_000); // Générer des seeds aléatoires
        }
        seed
    }

    fn load_cuda_functions(
        dev: &Arc<CudaDevice>,
        kernel: &CudaKernel,
        key: &str,
    ) -> HashMap<&'static str, CudaFunction> {
        let start = std::time::Instant::now();
        println!("Compiling CUDA kernels for {}", key);
        let ptx = compile_ptx(kernel.src).expect("Cuda Kernel failed to compile");
        dev.load_ptx(ptx, key, &kernel.funcs)
            .expect("Failed to load CUDA functions");
        let funcs = kernel
            .funcs
            .iter()
            .map(|&name| (name, dev.get_func(key, name).unwrap()))
            .collect();
        println!(
            "CUDA kernels for '{}' compiled in {}ms",
            key,
            start.elapsed().as_millis()
        );
        funcs
    }

    #[test]
    fn test_cuda_optimax_search_cuda() {
        let dev = CudaDevice::new(0).expect("Failed to create CudaDevice");
        let mut challenge_cuda_funcs = match &vector_search::KERNEL {
            Some(kernel) => load_cuda_functions(&dev, &kernel, "challenge"),
            None => {
                println!("No CUDA kernel for challenge");
                HashMap::new()
            }
        };
        let algorithm_cuda_funcs = match &optimax_gpu::KERNEL {
            Some(kernel) => load_cuda_functions(&dev, &kernel, "algorithm"),
            None => {
                println!("No CUDA kernel for algorithm");
                HashMap::new()
            }
        };

       // Générer une difficulté aléatoire et une seed aléatoire
       let mut i = 0;

       while i < 30 {
           let difficulty = random_difficulty();
           let seeds = random_seed();
           let challenge_cpu = Challenge::generate_instance(seeds, &difficulty).unwrap();

           println!(
               "Running test with difficulty: num_queries = {}, better_than_baseline = {}, nonce {:#?}",
               difficulty.num_queries, difficulty.better_than_baseline, seeds
           );


           let start_time = Instant::now();
            let result_current = optimax_gpu::solve_challenge(&challenge_cpu);
            let duration_current = start_time.elapsed();

            match result_current {
                Ok(Some(solution)) => match challenge_cpu.verify_solution(&solution) {
                    Ok(_) => println!("Valid solution (solve_challenge) ... ok"),
                    Err(e) => println!("Invalid solution (solve_challenge): {}", e),
                },
                Ok(None) => println!("No solution (solve_challenge)"),
                Err(e) => println!("Algorithm error (solve_challenge): {}", e),
            };

        let challenge =
            Challenge::cuda_generate_instance(seeds, &difficulty, &dev, challenge_cuda_funcs.clone())
                .unwrap();
        let start_time = Instant::now();
        let result_current_gpu = optimax_gpu::cuda_solve_challenge(&challenge, &dev, algorithm_cuda_funcs.clone());
        let duration_current_gpu = start_time.elapsed();
        match result_current_gpu {
            Ok(Some(solution)) => match challenge.verify_solution(&solution) {
                Ok(_) => println!("Valid solution"),
                Err(e) => println!("Invalid solution: {}", e),
            },
            Ok(None) => println!("No solution"),
            Err(e) => println!("Algorithm error: {}", e),
        };

         // Afficher les résultats de la comparaison de performance
         println!("Performance comparison:");
         println!("solve_challenge (cpu) duration: {:?}", duration_current);
         println!("solve_challenge_test (cuda) duration: {:?}", duration_current_gpu);

        i += 1;
    }
    }
}
// c004_a001

// c004_a002

// c004_a003

// c004_a004

// c004_a005

// c004_a006

// c004_a007

// c004_a008

// c004_a009

// c004_a010

// c004_a011

// c004_a012

// c004_a013

pub mod brute_force_bacalhau;
pub use brute_force_bacalhau as c004_a014;

// c004_a015

pub mod fast_search;
pub use fast_search as c004_a016;

// c004_a017

// c004_a018

// c004_a019

// c004_a020

// c004_a021

// c004_a022

// c004_a023

// c004_a024

// c004_a025

// c004_a027

// c004_a028

// c004_a029

// c004_a030

// c004_a031

// c004_a032

// c004_a033

// c004_a034

// c004_a035

// c004_a036

// c004_a037

// c004_a038

// c004_a039

// c004_a040

// c004_a041

// c004_a042

// c004_a043

// c004_a044

// c004_a045

// c004_a046

// c004_a047

// c004_a048

// c004_a049

// c004_a050

// c004_a051

// c004_a052

// c004_a053

// c004_a054

// c004_a055

// c004_a056

// c004_a057

// c004_a058

// c004_a059

// c004_a060

// c004_a061

// c004_a062

// c004_a063

// c004_a064

// c004_a065

// c004_a066

// c004_a067

// c004_a068

// c004_a069

// c004_a070

// c004_a071

// c004_a072

// c004_a073

// c004_a074

// c004_a075

// c004_a076

// c004_a077

// c004_a078

// c004_a079

// c004_a080

// c004_a081

// c004_a082

// c004_a083

// c004_a084

// c004_a085

// c004_a086

// c004_a087

// c004_a088

// c004_a089

// c004_a090

// c004_a091

// c004_a092

// c004_a093

// c004_a094

// c004_a095

// c004_a096

// c004_a097

// c004_a098

// c004_a099

// c004_a100

// c004_a101

// c004_a102

// c004_a103

// c004_a104

// c004_a105

// c004_a106

// c004_a107

// c004_a108

// c004_a109

// c004_a110

// c004_a111

// c004_a112

// c004_a113

// c004_a114

// c004_a115

// c004_a116

// c004_a117

// c004_a118

// c004_a119

// c004_a120

// c004_a121

// c004_a122

// c004_a123

// c004_a124

// c004_a125

// c004_a126

// c004_a127

// c004_a128

// c004_a129

// c004_a130

// c004_a131

// c004_a132

// c004_a133

// c004_a134

// c004_a135

// c004_a136

// c004_a137

// c004_a138

// c004_a139

// c004_a140

// c004_a141

// c004_a142

// c004_a143

// c004_a144

// c004_a145

// c004_a146

// c004_a147

// c004_a148

// c004_a149

// c004_a150

// c004_a151

// c004_a152

// c004_a153

// c004_a154

// c004_a155

// c004_a156

// c004_a157

// c004_a158

// c004_a159

// c004_a160

// c004_a161

// c004_a162

// c004_a163

// c004_a164

// c004_a165

// c004_a166

// c004_a167

// c004_a168

// c004_a169

// c004_a170

// c004_a171

// c004_a172

// c004_a173

// c004_a174

// c004_a175

// c004_a176

// c004_a177

// c004_a178

// c004_a179

// c004_a180

// c004_a181

// c004_a182

// c004_a183

// c004_a184

// c004_a185

// c004_a186

// c004_a187

// c004_a188

// c004_a189

// c004_a190

// c004_a191

// c004_a192

// c004_a193

// c004_a194

// c004_a195

// c004_a196

// c004_a197

// c004_a198

// c004_a199

// c004_a200

// c004_a201

// c004_a202

// c004_a203

// c004_a204

// c004_a205

// c004_a206

// c004_a207

// c004_a208

// c004_a209

// c004_a210

// c004_a211

// c004_a212

// c004_a213

// c004_a214

// c004_a215

// c004_a216

// c004_a217

// c004_a218

// c004_a219

// c004_a220

// c004_a221

// c004_a222

// c004_a223

// c004_a224

// c004_a225

// c004_a226

// c004_a227

// c004_a228

// c004_a229

// c004_a230

// c004_a231

// c004_a232

// c004_a233

// c004_a234

// c004_a235

// c004_a236

// c004_a237

// c004_a238

// c004_a239

// c004_a240

// c004_a241

// c004_a242

// c004_a243

// c004_a244

// c004_a245

// c004_a246

// c004_a247

// c004_a248

// c004_a249

// c004_a250

// c004_a251

// c004_a252

// c004_a253

// c004_a254

// c004_a255

// c004_a256

// c004_a257

// c004_a258

// c004_a259

// c004_a260

// c004_a261

// c004_a262

// c004_a263

// c004_a264

// c004_a265

// c004_a266

// c004_a267

// c004_a268

// c004_a269

// c004_a270

// c004_a271

// c004_a272

// c004_a273

// c004_a274

// c004_a275

// c004_a276

// c004_a277

// c004_a278

// c004_a279

// c004_a280

// c004_a281

// c004_a282

// c004_a283

// c004_a284

// c004_a285

// c004_a286

// c004_a287

// c004_a288

// c004_a289

// c004_a290

// c004_a291

// c004_a292

// c004_a293

// c004_a294

// c004_a295

// c004_a296

// c004_a297

// c004_a298

// c004_a299

// c004_a300

// c004_a301

// c004_a302

// c004_a303

// c004_a304

// c004_a305

// c004_a306

// c004_a307

// c004_a308

// c004_a309

// c004_a310

// c004_a311

// c004_a312

// c004_a313

// c004_a314

// c004_a315

// c004_a316

// c004_a317

// c004_a318

// c004_a319

// c004_a320

// c004_a321

// c004_a322

// c004_a323

// c004_a324

// c004_a325

// c004_a326

// c004_a327

// c004_a328

// c004_a329

// c004_a330

// c004_a331

// c004_a332

// c004_a333

// c004_a334

// c004_a335

// c004_a336

// c004_a337

// c004_a338

// c004_a339

// c004_a340

// c004_a341

// c004_a342

// c004_a343

// c004_a344

// c004_a345

// c004_a346

// c004_a347

// c004_a348

// c004_a349

// c004_a350

// c004_a351

// c004_a352

// c004_a353

// c004_a354

// c004_a355

// c004_a356

// c004_a357

// c004_a358

// c004_a359

// c004_a360

// c004_a361

// c004_a362

// c004_a363

// c004_a364

// c004_a365

// c004_a366

// c004_a367

// c004_a368

// c004_a369

// c004_a370

// c004_a371

// c004_a372

// c004_a373

// c004_a374

// c004_a375

// c004_a376

// c004_a377

// c004_a378

// c004_a379

// c004_a380

// c004_a381

// c004_a382

// c004_a383

// c004_a384

// c004_a385

// c004_a386

// c004_a387

// c004_a388

// c004_a389

// c004_a390

// c004_a391

// c004_a392

// c004_a393

// c004_a394

// c004_a395

// c004_a396

// c004_a397

// c004_a398

// c004_a399

// c004_a400

// c004_a401

// c004_a402

// c004_a403

// c004_a404

// c004_a405

// c004_a406

// c004_a407

// c004_a408

// c004_a409

// c004_a410

// c004_a411

// c004_a412

// c004_a413

// c004_a414

// c004_a415

// c004_a416

// c004_a417

// c004_a418

// c004_a419

// c004_a420

// c004_a421

// c004_a422

// c004_a423

// c004_a424

// c004_a425

// c004_a426

// c004_a427

// c004_a428

// c004_a429

// c004_a430

// c004_a431

// c004_a432

// c004_a433

// c004_a434

// c004_a435

// c004_a436

// c004_a437

// c004_a438

// c004_a439

// c004_a440

// c004_a441

// c004_a442

// c004_a443

// c004_a444

// c004_a445

// c004_a446

// c004_a447

// c004_a448

// c004_a449

// c004_a450

// c004_a451

// c004_a452

// c004_a453

// c004_a454

// c004_a455

// c004_a456

// c004_a457

// c004_a458

// c004_a459

// c004_a460

// c004_a461

// c004_a462

// c004_a463

// c004_a464

// c004_a465

// c004_a466

// c004_a467

// c004_a468

// c004_a469

// c004_a470

// c004_a471

// c004_a472

// c004_a473

// c004_a474

// c004_a475

// c004_a476

// c004_a477

// c004_a478

// c004_a479

// c004_a480

// c004_a481

// c004_a482

// c004_a483

// c004_a484

// c004_a485

// c004_a486

// c004_a487

// c004_a488

// c004_a489

// c004_a490

// c004_a491

// c004_a492

// c004_a493

// c004_a494

// c004_a495

// c004_a496

// c004_a497

// c004_a498

// c004_a499

// c004_a500

// c004_a501

// c004_a502

// c004_a503

// c004_a504

// c004_a505

// c004_a506

// c004_a507

// c004_a508

// c004_a509

// c004_a510

// c004_a511

// c004_a512

// c004_a513

// c004_a514

// c004_a515

// c004_a516

// c004_a517

// c004_a518

// c004_a519

// c004_a520

// c004_a521

// c004_a522

// c004_a523

// c004_a524

// c004_a525

// c004_a526

// c004_a527

// c004_a528

// c004_a529

// c004_a530

// c004_a531

// c004_a532

// c004_a533

// c004_a534

// c004_a535

// c004_a536

// c004_a537

// c004_a538

// c004_a539

// c004_a540

// c004_a541

// c004_a542

// c004_a543

// c004_a544

// c004_a545

// c004_a546

// c004_a547

// c004_a548

// c004_a549

// c004_a550

// c004_a551

// c004_a552

// c004_a553

// c004_a554

// c004_a555

// c004_a556

// c004_a557

// c004_a558

// c004_a559

// c004_a560

// c004_a561

// c004_a562

// c004_a563

// c004_a564

// c004_a565

// c004_a566

// c004_a567

// c004_a568

// c004_a569

// c004_a570

// c004_a571

// c004_a572

// c004_a573

// c004_a574

// c004_a575

// c004_a576

// c004_a577

// c004_a578

// c004_a579

// c004_a580

// c004_a581

// c004_a582

// c004_a583

// c004_a584

// c004_a585

// c004_a586

// c004_a587

// c004_a588

// c004_a589

// c004_a590

// c004_a591

// c004_a592

// c004_a593

// c004_a594

// c004_a595

// c004_a596

// c004_a597

// c004_a598

// c004_a599

// c004_a600

// c004_a601

// c004_a602

// c004_a603

// c004_a604

// c004_a605

// c004_a606

// c004_a607

// c004_a608

// c004_a609

// c004_a610

// c004_a611

// c004_a612

// c004_a613

// c004_a614

// c004_a615

// c004_a616

// c004_a617

// c004_a618

// c004_a619

// c004_a620

// c004_a621

// c004_a622

// c004_a623

// c004_a624

// c004_a625

// c004_a626

// c004_a627

// c004_a628

// c004_a629

// c004_a630

// c004_a631

// c004_a632

// c004_a633

// c004_a634

// c004_a635

// c004_a636

// c004_a637

// c004_a638

// c004_a639

// c004_a640

// c004_a641

// c004_a642

// c004_a643

// c004_a644

// c004_a645

// c004_a646

// c004_a647

// c004_a648

// c004_a649

// c004_a650

// c004_a651

// c004_a652

// c004_a653

// c004_a654

// c004_a655

// c004_a656

// c004_a657

// c004_a658

// c004_a659

// c004_a660

// c004_a661

// c004_a662

// c004_a663

// c004_a664

// c004_a665

// c004_a666

// c004_a667

// c004_a668

// c004_a669

// c004_a670

// c004_a671

// c004_a672

// c004_a673

// c004_a674

// c004_a675

// c004_a676

// c004_a677

// c004_a678

// c004_a679

// c004_a680

// c004_a681

// c004_a682

// c004_a683

// c004_a684

// c004_a685

// c004_a686

// c004_a687

// c004_a688

// c004_a689

// c004_a690

// c004_a691

// c004_a692

// c004_a693

// c004_a694

// c004_a695

// c004_a696

// c004_a697

// c004_a698

// c004_a699

// c004_a700

// c004_a701

// c004_a702

// c004_a703

// c004_a704

// c004_a705

// c004_a706

// c004_a707

// c004_a708

// c004_a709

// c004_a710

// c004_a711

// c004_a712

// c004_a713

// c004_a714

// c004_a715

// c004_a716

// c004_a717

// c004_a718

// c004_a719

// c004_a720

// c004_a721

// c004_a722

// c004_a723

// c004_a724

// c004_a725

// c004_a726

// c004_a727

// c004_a728

// c004_a729

// c004_a730

// c004_a731

// c004_a732

// c004_a733

// c004_a734

// c004_a735

// c004_a736

// c004_a737

// c004_a738

// c004_a739

// c004_a740

// c004_a741

// c004_a742

// c004_a743

// c004_a744

// c004_a745

// c004_a746

// c004_a747

// c004_a748

// c004_a749

// c004_a750

// c004_a751

// c004_a752

// c004_a753

// c004_a754

// c004_a755

// c004_a756

// c004_a757

// c004_a758

// c004_a759

// c004_a760

// c004_a761

// c004_a762

// c004_a763

// c004_a764

// c004_a765

// c004_a766

// c004_a767

// c004_a768

// c004_a769

// c004_a770

// c004_a771

// c004_a772

// c004_a773

// c004_a774

// c004_a775

// c004_a776

// c004_a777

// c004_a778

// c004_a779

// c004_a780

// c004_a781

// c004_a782

// c004_a783

// c004_a784

// c004_a785

// c004_a786

// c004_a787

// c004_a788

// c004_a789

// c004_a790

// c004_a791

// c004_a792

// c004_a793

// c004_a794

// c004_a795

// c004_a796

// c004_a797

// c004_a798

// c004_a799

// c004_a800

// c004_a801

// c004_a802

// c004_a803

// c004_a804

// c004_a805

// c004_a806

// c004_a807

// c004_a808

// c004_a809

// c004_a810

// c004_a811

// c004_a812

// c004_a813

// c004_a814

// c004_a815

// c004_a816

// c004_a817

// c004_a818

// c004_a819

// c004_a820

// c004_a821

// c004_a822

// c004_a823

// c004_a824

// c004_a825

// c004_a826

// c004_a827

// c004_a828

// c004_a829

// c004_a830

// c004_a831

// c004_a832

// c004_a833

// c004_a834

// c004_a835

// c004_a836

// c004_a837

// c004_a838

// c004_a839

// c004_a840

// c004_a841

// c004_a842

// c004_a843

// c004_a844

// c004_a845

// c004_a846

// c004_a847

// c004_a848

// c004_a849

// c004_a850

// c004_a851

// c004_a852

// c004_a853

// c004_a854

// c004_a855

// c004_a856

// c004_a857

// c004_a858

// c004_a859

// c004_a860

// c004_a861

// c004_a862

// c004_a863

// c004_a864

// c004_a865

// c004_a866

// c004_a867

// c004_a868

// c004_a869

// c004_a870

// c004_a871

// c004_a872

// c004_a873

// c004_a874

// c004_a875

// c004_a876

// c004_a877

// c004_a878

// c004_a879

// c004_a880

// c004_a881

// c004_a882

// c004_a883

// c004_a884

// c004_a885

// c004_a886

// c004_a887

// c004_a888

// c004_a889

// c004_a890

// c004_a891

// c004_a892

// c004_a893

// c004_a894

// c004_a895

// c004_a896

// c004_a897

// c004_a898

// c004_a899

// c004_a900

// c004_a901

// c004_a902

// c004_a903

// c004_a904

// c004_a905

// c004_a906

// c004_a907

// c004_a908

// c004_a909

// c004_a910

// c004_a911

// c004_a912

// c004_a913

// c004_a914

// c004_a915

// c004_a916

// c004_a917

// c004_a918

// c004_a919

// c004_a920

// c004_a921

// c004_a922

// c004_a923

// c004_a924

// c004_a925

// c004_a926

// c004_a927

// c004_a928

// c004_a929

// c004_a930

// c004_a931

// c004_a932

// c004_a933

// c004_a934

// c004_a935

// c004_a936

// c004_a937

// c004_a938

// c004_a939

// c004_a940

// c004_a941

// c004_a942

// c004_a943

// c004_a944

// c004_a945

// c004_a946

// c004_a947

// c004_a948

// c004_a949

// c004_a950

// c004_a951

// c004_a952

// c004_a953

// c004_a954

// c004_a955

// c004_a956

// c004_a957

// c004_a958

// c004_a959

// c004_a960

// c004_a961

// c004_a962

// c004_a963

// c004_a964

// c004_a965

// c004_a966

// c004_a967

// c004_a968

// c004_a969

// c004_a970

// c004_a971

// c004_a972

// c004_a973

// c004_a974

// c004_a975

// c004_a976

// c004_a977

// c004_a978

// c004_a979

// c004_a980

// c004_a981

// c004_a982

// c004_a983

// c004_a984

// c004_a985

// c004_a986

// c004_a987

// c004_a988

// c004_a989

// c004_a990

// c004_a991

// c004_a992

// c004_a993

// c004_a994

// c004_a995

// c004_a996

// c004_a997

// c004_a998

// c004_a999
