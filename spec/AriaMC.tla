----------------------------- MODULE AriaMC --------------------------------
(***************************************************************************
  Finite model instance of Aria for TLC model checking in principle.

  Instantiates abstract carriers with small finite sets and operators that
  satisfy the ASSUME clauses of Aria.tla (ideal unitarity, total maps,
  GraphOK seed, Dist(x,x)=0, etc.).

  Does not invent architecture: only supplies a finite discrete image of
  the domains already fixed in the documentation.
 ***************************************************************************)

EXTENDS Integers, FiniteSets, TLC

\* ---- Finite carriers ----
N_MC    == 2
M0_MC   == 2
MaxT_MC == 3

Hilbert_MC == {"h0", "h1"}
Latent_MC  == {"z0", "z1"}
Cond_MC    == {"a0", "a1"}
NodeId_MC  == {"n1", "n2"}
EdgeType_MC == {"morph"}

Psi0_MC == "h0"

EmptyEmb == [n \in NodeId_MC |-> "z0"]

G0_MC ==
  [nodes |-> {},
   edges |-> {},
   emb   |-> EmptyEmb]

GStar_MC ==
  [nodes |-> {"n1"},
   edges |-> {},
   emb   |-> [EmptyEmb EXCEPT !["n1"] = "z0"]]

Eps_MC == 1

\* 𝔸2 — isometry image (injective on this tiny model)
Embed_MC(psi) ==
  IF psi = "h0" THEN "z0" ELSE "z1"

\* 𝔸4 — unitary family: permutation of Hilbert (energy-preserving)
UOp_MC(tt, psi) ==
  IF psi = "h0" THEN "h1" ELSE "h0"

\* Energy: constant on modes (‖ψ‖₂ abstracted to 1 for all unit fields)
Energy_MC(psi) == 1

\* Predictor: identity-like contractive map on latents
Pred_MC(zz, c) == zz

\* Diff: flip latent (graph-conditioned abstractly; ignores g for MC size)
DiffOp_MC(g, zz) ==
  IF zz = "z0" THEN "z1" ELSE "z0"

\* Distance
Dist_MC(x, y) ==
  IF x = y THEN 0 ELSE 1

\* Conditioning schedule (𝐂2: change of a_t only)
AOf_MC(tt) ==
  IF tt % 2 = 0 THEN "a0" ELSE "a1"

\* True future embeddings (for optional JEPA temporal checks)
TrueZ_MC(tt) ==
  IF tt % 2 = 0 THEN "z0" ELSE "z1"

====
