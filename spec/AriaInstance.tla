------------------------- MODULE AriaInstance ------------------------------
(***************************************************************************
  Runnable TLC entry point: finite INSTANCE of Aria.

    java -cp tla2tools.jar tlc2.TLC -config AriaInstance.cfg AriaInstance

  Constants and operators come from AriaMC; variables are local and
  substituted into Aria (SANY requires explicit VARIABLE substitution).
 ***************************************************************************)

EXTENDS AriaMC

VARIABLES psi, z, G, t, prevRes

INSTANCE Aria WITH
  N <- N_MC,
  M0 <- M0_MC,
  MaxT <- MaxT_MC,
  Hilbert <- Hilbert_MC,
  Latent <- Latent_MC,
  Cond <- Cond_MC,
  NodeId <- NodeId_MC,
  EdgeType <- EdgeType_MC,
  Psi0 <- Psi0_MC,
  G0 <- G0_MC,
  GStar <- GStar_MC,
  Eps <- Eps_MC,
  Embed <- Embed_MC,
  UOp <- UOp_MC,
  Pred <- Pred_MC,
  DiffOp <- DiffOp_MC,
  Energy <- Energy_MC,
  Dist <- Dist_MC,
  AOf <- AOf_MC,
  TrueZ <- TrueZ_MC,
  psi <- psi,
  z <- z,
  G <- G,
  t <- t,
  prevRes <- prevRes

\* Finite-state bound for TLC only (abstract Spec: t ∈ Nat, unbounded).
StateConstraint == t <= MaxT_MC

====

