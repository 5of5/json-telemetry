------------------------------ MODULE Aria --------------------------------
(***************************************************************************
  Aria — Ariadne Transformer
  Self-contained TLA+ specification of admissible discrete behaviors.

  Faithful encoding of:
    docs/FORMAL_SPEC.md, docs/SAFETY.md, docs/CONTINUOUS_REFINEMENT.md,
    docs/ASYMPTOTICS.md, docs/RATIONALE.md, README.md
  Validation: docs/VALIDATION.md

  This is not a program. It is a specification of allowed infinite
  sequences of states. Continuous optical/latent machinery appears only
  as sources of discrete transitions (see CONTINUOUS_REFINEMENT.md).
  No training recipe. No hardware.

  Grain of atomicity: one complete Φ-step
      Φ = Diff ∘ Match ∘ P ∘ U
  realized as the disjunction of four named actions + stuttering.
 ***************************************************************************)

EXTENDS Integers, FiniteSets, Sequences, TLC

(***************************************************************************
 * Step 2 — Deductive anchors (comments; operationalized by ASSUME below)
 *
 * 𝔸1  Hilbert space of N modes → finite carrier Hilbert
 * 𝔸2  isometry I : H ↪ Z          → operator Embed
 * 𝔸3  experience graph typed        → GraphOK
 * 𝔸4  optical maps (near-)unitary   → Energy(U(t,ψ)) = Energy(ψ)
 * ℙ1  optical similarity depth O(log N)     → asymptotic consequence only
 * ℙ2  E[Lip(P)] ≤ 1                 → residual tolerance Eps
 * ℙ3  ED = finite elementary edits  → Match preserves GraphOK
 * 𝐋1–𝐋3, 𝐓1–𝐓2, 𝐂1–𝐂3             → winning / asymptotics (comments + defs)
 ***************************************************************************)

CONSTANTS
  N,              \* number of optical modes (𝔸1), N ≥ 1
  M0,             \* initial bound on |V(G0)|
  Hilbert,        \* finite abstract carrier of optical field states
  Latent,         \* finite abstract carrier of JEPA latents
  Cond,           \* finite set of conditionings a_t (𝐂2)
  NodeId,         \* finite pool of graph node identifiers
  EdgeType,       \* finite set of typed morphisms (𝔸3)
  Psi0,           \* initial optical field
  G0,             \* empty or seed graph (record)
  GStar,          \* target/expert graph for Match
  Eps,            \* contractivity tolerance ε ≥ 0 (ℙ2)
  Embed(_),       \* I : Hilbert → Latent (𝔸2)
  UOp(_, _),      \* U(t, ψ) : unitary optical step (𝔸4)
  Pred(_, _),     \* P(latent, cond) (ℙ2)
  DiffOp(_, _),   \* Diff_G(z)
  Energy(_),      \* abstract ‖·‖₂ (Inv1)
  Dist(_, _),     \* latent distance (Inv2)
  AOf(_),         \* a(t) conditioning schedule
  TrueZ(_),       \* true future embedding at step t (winning JEPA only)
  MaxT            \* optional bound for model checking (may be used by cfg)

(***************************************************************************
 * Graph representation (𝔸3)
 *   nodes ⊆ NodeId
 *   edges ⊆ NodeId × NodeId × EdgeType
 *   emb   : [NodeId → Latent]  (values matter only on nodes)
 ***************************************************************************)

IsGraphShape(g) ==
  /\ g = [nodes |-> g.nodes, edges |-> g.edges, emb |-> g.emb]
  /\ g.nodes \subseteq NodeId
  /\ g.edges \subseteq (NodeId \X NodeId \X EdgeType)
  /\ g.emb \in [NodeId -> Latent]

\* Every edge endpoint is a node; embeddings on nodes are in Latent (always
\* true by emb's type); edges are typed morphisms by construction of EdgeType.
GraphOK(g) ==
  /\ IsGraphShape(g)
  /\ IsFiniteSet(g.nodes)
  /\ \A e \in g.edges :
        /\ e[1] \in g.nodes
        /\ e[2] \in g.nodes

(***************************************************************************
 * ASSUME — axiomatic constraints (𝔸, ℙ) that TLC instantiations must obey.
 * A concrete model (AriaMC) supplies finite sets and operators that satisfy
 * these assumptions so that the Spec remains checkable in principle.
 ***************************************************************************)

ASSUME N \in Nat \ {0}
ASSUME M0 \in Nat
ASSUME Eps \in Nat
ASSUME Psi0 \in Hilbert
ASSUME IsFiniteSet(Hilbert) /\ Hilbert # {}
ASSUME IsFiniteSet(Latent)  /\ Latent  # {}
ASSUME IsFiniteSet(Cond)    /\ Cond    # {}
ASSUME IsFiniteSet(NodeId)
ASSUME IsFiniteSet(EdgeType)
ASSUME GraphOK(G0)
ASSUME Cardinality(G0.nodes) <= M0
ASSUME GraphOK(GStar)

\* 𝔸2 — embedding total into Latent
ASSUME \A psi \in Hilbert : Embed(psi) \in Latent

\* 𝔸4 — ideal lossless unitarity: Energy preserved by every U(t,·)
ASSUME \A t \in Nat, psi \in Hilbert :
          /\ UOp(t, psi) \in Hilbert
          /\ Energy(UOp(t, psi)) = Energy(psi)

ASSUME \A psi \in Hilbert : Energy(psi) \in Nat

\* ℙ2 — predictor total; residual discipline is enforced by action obligations
ASSUME \A zz \in Latent, c \in Cond : Pred(zz, c) \in Latent

\* Diff total on GraphOK × Latent (instance obligation; 𝔸3 / Diffuse)
\* TLC checks totality on reachable graphs via TypeOK of post-states.
ASSUME \A zz \in Latent :
          DiffOp(G0, zz) \in Latent

\* Distance is a Nat-valued measure (non-negativity + zero diagonal)
ASSUME \A x \in Latent, y \in Latent : Dist(x, y) \in Nat
ASSUME \A x \in Latent : Dist(x, x) = 0

\* Conditioning schedule and true future (for winning property only)
ASSUME \A tt \in Nat : AOf(tt) \in Cond
ASSUME \A tt \in Nat : TrueZ(tt) \in Latent

(***************************************************************************
 * Step 3 — VARIABLES
 ***************************************************************************)

VARIABLES
  psi,       \* optical field ∈ Hilbert
  z,         \* JEPA latent ∈ Latent
  G,         \* experience/thought graph
  t,         \* discrete step counter ∈ Nat
  prevRes    \* auxiliary history: previous Res (Inv2 as in SAFETY.md)

vars == <<psi, z, G, t, prevRes>>

\* Observable documentation tuple (excludes pure history)
obsVars == <<psi, z, G, t>>

(***************************************************************************
 * Derived operators
 ***************************************************************************)

Res(p, zz, tt) == Dist(zz, Pred(Embed(p), AOf(tt)))

\* G ∪ {z}: add one fresh node carrying embedding zz, if capacity remains;
\* if NodeId is exhausted, leave G unchanged (finite-model totality — does
\* not enlarge documentation behaviors beyond typed finite graphs).
FreshNode(g) ==
  IF \E n \in NodeId : n \notin g.nodes
  THEN CHOOSE n \in NodeId : n \notin g.nodes
  ELSE CHOOSE n \in NodeId : TRUE  \* dummy; AddNodeZ will no-op if full

AddNodeZ(g, zz) ==
  IF \E n \in NodeId : n \notin g.nodes
  THEN LET n == FreshNode(g) IN
       [nodes |-> g.nodes \cup {n},
        edges |-> g.edges,
        emb   |-> [g.emb EXCEPT ![n] = zz]]
  ELSE g

(***************************************************************************
 * ℙ3 — Elementary edits and ED
 * ED(g, gStar) is any finite composition of Add/Delete/Relabel that yields
 * a GraphOK result. For the Spec we abstract ED as the nondeterministic
 * choice of a GraphOK graph reachable by elementary edits from g, with
 * the documentation intent of matching toward GStar.
 *
 * To remain checkable and not invent architecture, ED is specified as:
 *   GraphOK(g') ∧ EditReachable(g, g')
 * where EditReachable is the reflexive-transitive closure of one elementary
 * edit. For TLC finiteness we bound the definition to a single elementary
 * edit OR identity OR direct jump to GStar (still GraphOK) — all of which
 * are instances of finite elementary-edit sequences (including empty / full
 * rebuild as a finite sequence of deletes+adds). Rebuild-to-GStar is a
 * finite elementary sequence and is admitted by ℙ3.
 ***************************************************************************)

ElementaryEdit(g, g2) ==
  /\ GraphOK(g) /\ GraphOK(g2)
  /\ \/ \* identity (empty edit sequence)
        g2 = g
     \/ \* add one node
        /\ \E n \in NodeId :
             /\ n \notin g.nodes
             /\ g2.nodes = g.nodes \cup {n}
             /\ g2.edges = g.edges
             /\ \E zz \in Latent : g2.emb = [g.emb EXCEPT ![n] = zz]
     \/ \* delete one node and incident edges
        /\ \E n \in g.nodes :
             /\ g2.nodes = g.nodes \ {n}
             /\ g2.edges = { e \in g.edges : e[1] # n /\ e[2] # n }
             /\ g2.emb = g.emb
     \/ \* relabel one node embedding
        /\ g2.nodes = g.nodes
        /\ g2.edges = g.edges
        /\ \E n \in g.nodes, zz \in Latent :
             g2.emb = [g.emb EXCEPT ![n] = zz]
     \/ \* add one typed edge
        /\ g2.nodes = g.nodes
        /\ g2.emb = g.emb
        /\ \E e \in (NodeId \X NodeId \X EdgeType) :
             /\ e[1] \in g.nodes /\ e[2] \in g.nodes
             /\ e \notin g.edges
             /\ g2.edges = g.edges \cup {e}
     \/ \* delete one edge
        /\ g2.nodes = g.nodes
        /\ g2.emb = g.emb
        /\ \E e \in g.edges : g2.edges = g.edges \ {e}
     \/ \* finite rebuild to GStar (finite sequence of elementary edits)
        g2 = GStar

\* Finite set of one-step elementary results from g (ℙ3), plus identity and G*.
\* Used by Match so TLC does not enumerate the full Graph carrier.
OneStepEdits(g) ==
  {g, GStar} \cup
  { [nodes |-> g.nodes \cup {n},
     edges |-> g.edges,
     emb   |-> [g.emb EXCEPT ![n] = zz]]
  : n \in NodeId, zz \in Latent } \cup
  { [nodes |-> g.nodes \ {n},
     edges |-> { e \in g.edges : e[1] # n /\ e[2] # n },
     emb   |-> g.emb]
  : n \in g.nodes } \cup
  { [nodes |-> g.nodes,
     edges |-> g.edges,
     emb   |-> [g.emb EXCEPT ![n] = zz]]
  : n \in g.nodes, zz \in Latent } \cup
  { [nodes |-> g.nodes,
     edges |-> g.edges \cup {e},
     emb   |-> g.emb]
  : e \in (g.nodes \X g.nodes \X EdgeType) } \cup
  { [nodes |-> g.nodes,
     edges |-> g.edges \ {e},
     emb   |-> g.emb]
  : e \in g.edges }

ED(g, gStar) == ElementaryEdit(g, gStar)  \* documentation name for ℙ3

(***************************************************************************
 * Step 4 — TypeOK
 ***************************************************************************)

TypeOK ==
  /\ psi \in Hilbert
  /\ z \in Latent
  /\ GraphOK(G)
  /\ t \in Nat
  /\ prevRes \in Nat

(***************************************************************************
 * Step 5 — Init
 * Proof Init ⇒ TypeOK ∧ Inv1..Inv4 : see docs/FORMAL_SPEC.md and docs/SAFETY.md
 ***************************************************************************)

Init ==
  /\ psi = Psi0
  /\ z = Embed(Psi0)
  /\ G = G0
  /\ t = 0
  /\ prevRes = Dist(Embed(Psi0), Pred(Embed(Psi0), AOf(0)))

(***************************************************************************
 * Step 6 — Named actions realizing Φ
 *
 * Common history obligation (names SAFETY.md “prev” residual):
 *   prevRes' = Res(psi, z, t)
 * Contractivity obligation on the post-state residual:
 *   Res(psi', z', t') ≤ Res(psi, z, t) + Eps
 ***************************************************************************)

\* ----- 6.1 OpticalStep -----
\* Enabling: TypeOK (implicit under Spec reachable states)
\* Updates: ψ' = U(t, ψ)
\* UNCHANGED: <<z, G, t>>
OpticalStep ==
  /\ psi' = UOp(t, psi)
  /\ UNCHANGED <<z, G, t>>
  /\ prevRes' = Res(psi, z, t)
  /\ Res(psi', z', t') <= Res(psi, z, t) + Eps
  \* Inv1 preservation follows from ASSUME on UOp (Energy equal).
  \* The residual inequality is the discrete image of ℙ2 under field change.

\* ----- 6.2 Predict -----
\* z' = P(I(ψ), a_t); UNCHANGED <<ψ, G, t>>
Predict ==
  /\ z' = Pred(Embed(psi), AOf(t))
  /\ UNCHANGED <<psi, G, t>>
  /\ prevRes' = Res(psi, z, t)
  /\ Res(psi', z', t') <= Res(psi, z, t) + Eps
  \* After Predict, Res(psi',z',t') = Dist(Pred(...), Pred(...)) = 0.

\* ----- 6.3 Match -----
\* G' = ED(G ∪ {z}, G*); UNCHANGED <<ψ, z, t>>
\* ED realized by one elementary-edit step (or identity / rebuild to G*)
\* from G⊕z, per ℙ3 (OneStepEdits); no full Graph-carrier enumeration.
Match ==
  /\ \E g2 \in OneStepEdits(AddNodeZ(G, z)) :
        /\ GraphOK(g2)
        /\ G' = g2
  /\ UNCHANGED <<psi, z, t>>
  /\ prevRes' = Res(psi, z, t)
  /\ Res(psi', z', t') <= Res(psi, z, t) + Eps
  \* Res unchanged because <<psi,z,t>> unchanged; inequality holds with 0 ≤ Eps.

\* ----- 6.4 Diffuse -----
\* z' = Diff_G(z); t' = t+1; UNCHANGED <<ψ, G>>
Diffuse ==
  /\ z' = DiffOp(G, z)
  /\ t' = t + 1
  /\ UNCHANGED <<psi, G>>
  /\ prevRes' = Res(psi, z, t)
  /\ Res(psi', z', t') <= Res(psi, z, t) + Eps

\* ----- Stuttering (permitted) -----
Stutter ==
  UNCHANGED vars

(***************************************************************************
 * Step 7 — Next
 ***************************************************************************)

Next ==
  \/ OpticalStep
  \/ Predict
  \/ Match
  \/ Diffuse
  \/ Stutter

(***************************************************************************
 * Step 8 — Spec
 ***************************************************************************)

Spec ==
  Init /\ [][Next]_vars

(***************************************************************************
 * Step 9 — Primary safety invariants
 ***************************************************************************)

Inv1 == Energy(psi) = Energy(Psi0)

Inv2 == Res(psi, z, t) <= prevRes + Eps

Inv3 == GraphOK(G)

Inv4 == TypeOK

Safety == Inv1 /\ Inv2 /\ Inv3 /\ Inv4

\* Joint inductiveness: Spec ⇒ □ Safety
\* Base and step arguments: docs/FORMAL_SPEC.md and docs/SAFETY.md
THEOREM Spec => []Safety
\* Proof is by standard TLA invariance (Init⇒Safety; Safety∧[Next]_vars⇒Safety')
\* and ASSUME (𝔸4, ℙ2, ℙ3) as discharged in the action obligations above.
PROOF OMITTED

(***************************************************************************
 * Step 10 — Winning condition (SAFETY.md §3)
 *
 * Winning is Spec ∧ □Safety plus liveness/asymptotic clauses. TLC does not
 * check “tends to zero”; we encode the JEPA clause as a temporal property
 * schema and the addressability clause as a constant assertion from 𝐋1.
 ***************************************************************************)

\* Joint embedding predictive property (schema): residual to true future
\* eventually stays below every positive bound K (limit to 0 along the path).
JEPALimit(K) ==
  <>[](Dist(Pred(Embed(psi), AOf(t)), TrueZ(t)) <= K)

\* Optical addressability in O(1) transit (𝐋1, 𝐂3): not a state variable;
\* recorded as a specification axiom of the optical substrate, not checked
\* by state exploration.
OpticalAddressable == TRUE   \* justified by 𝐋1; see docs/ASYMPTOTICS.md

\* Full winning condition for a fixed K (model-checker approximation of →0)
Winning(K) ==
  /\ Spec
  /\ []Safety
  /\ JEPALimit(K)
  /\ OpticalAddressable

(***************************************************************************
 * Step 11 — Asymptotic corollaries (consequences of Spec structure only)
 * Not part of the transition system. Documented for the record.
 *
 *   depth(Φ-step)     = O(log N + polylog M)     from ℙ1
 *   energy / MAC      = O(N^{-1})                 from Inv1, 𝔸1, 𝔸4
 *   |G| after T traj. = O(T^β), β ≤ 1            from 𝐋3
 *   ranking latency   = O(1) optical             from 𝐋1 → 𝐂3
 ***************************************************************************)

====
