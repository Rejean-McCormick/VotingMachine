# **Annex B — Part 0: Schema & Conventions**

**Scope.** Defines the common rules, identifiers, and shapes used by all test fixtures in Annex B (Parts 1–7). Aligns with Docs 1–7.

---

## **1\) Identifier patterns (canonical)**

* **DivisionRegistry ID:** `REG:<name>:<version>`

* **Unit ID:** `U:<REG_ID>:<path>` (tree path; root has no parent)

* **Option ID:** `OPT:<slug>`

* **BallotTally ID:** `TLY:<name>:v<digit>`

* **ParameterSet ID:** `PS:<name>:v<digit>`

* **Result ID:** `RES:<hash>`

* **RunRecord ID:** `RUN:<timestamp>-<hash>` (format per Doc 1B)

* **FrontierMap ID:** `FR:<hash>`

* **AutonomyPackage ID:** `AP:<name>:v<digit>`

**Ordering rules (deterministic):**

* Units sorted lexicographically by **Unit ID**.

* Options sorted by **Option.order\_index** (ascending), then by **Option ID**.

* All lists in fixtures should already respect these orders.

---

## **2\) Core conventions**

* **Percent parameters** are integers `0..100`. Internals use exact integer/rational math.

* **Approval gate denominator (fixed):** for approval ballots, **support % \= approvals\_for\_change / valid ballots**.

* **Rounding (internal):** *round half to even* only at defined comparison points (Docs 4A/4C).

* **Rounding (presentation):** one decimal place in expected values shown in Annex B and reports (Doc 7).

* **Determinism:** same inputs \+ same seed ⇒ byte-identical **Result** and **RunRecord**.

* **RNG use:** only if `tie_policy = random`; algorithm is the seeded stream RNG defined in Doc 3A; seed recorded as **VM-VAR-052**.

* **Offline:** fixtures assume no network access (Doc 3A).

---

## **3\) Validation rules expected of engines (applied to all parts)**

* **Hierarchy:** Units form a tree within one **DivisionRegistry**; exactly one root; no cycles.

* **Magnitude:** `Unit.magnitude ≥ 1`; if `VM-VAR-010 = winner_take_all`, every involved Unit must have `magnitude = 1`.

* **Tally sanity (per Unit):** `Σ(valid option tallies) + invalid_or_blank ≤ ballots_cast`.

* **Eligible roll:** if `VM-VAR-020 > 0` or `VM-VAR-021 > 0`, each aggregated Unit must have `eligible_roll ≥ ballots_cast`.

* **Population weighting:** if `VM-VAR-030 = population_baseline`, each aggregated Unit must have `population_baseline > 0` and a `population_baseline_year`.

* **Frontier bands:** if `VM-VAR-040 ∈ {sliding_scale, autonomy_ladder}`, bands (VM-VAR-042) are ordered, non-overlapping, and respect the intended ranges.

* **Contiguity types:** `Adjacency.type ∈ {land, bridge, water}`; `VM-VAR-047` is a subset; `VM-VAR-048 ∈ {none, ferry_allowed, corridor_required}`.

* **Double-majority w/o frontier:** if `VM-VAR-024 = on` and `VM-VAR-040 = none`, then `VM-VAR-026 ∈ {by_list, by_tag}` and `VM-VAR-027` resolves to a non-empty family.

---

## **4\) Fixture shapes (what each part will contain)**

### **4.1 DivisionRegistry**

* `id`, `provenance{source, published_date}`

* **Units\[\]:** `id`, `name`, `level`, optional `parent`, `magnitude`, `eligible_roll`, optional `population_baseline` & `population_baseline_year`, optional `protected_area`

* **Adjacency\[\] (optional):** `unit_id_a`, `unit_id_b`, `type` (`land|bridge|water`), optional `notes`

### **4.2 Options**

* **Options\[\]:** `id`, `display_name`, `order_index` (unique), `is_status_quo` (bool)

### **4.3 Ballot tallies (per ballot type)**

* **Approval:** per Unit: `ballots_cast`, `invalid_or_blank`, `approvals{Option→count}`

* **Plurality:** per Unit: `ballots_cast`, `invalid_or_blank`, `votes{Option→count}`

* **Score:** per Unit: `ballots_cast`, `invalid_or_blank`, `score_sum{Option→sum}`, `ballots_counted`; plus scale (`VM-VAR-002..003`) and normalization (`VM-VAR-004`)

* **Ranked IRV (executive or unit):** `rounds[{ranking[], count}]`; exhaustion policy is `reduce_continuing_denominator` (VM-VAR-006)

* **Ranked Condorcet:** `ballots[{ranking[], count}]`; completion rule per `VM-VAR-005`

### **4.4 ParameterSet**

* `id`; `vars{VM-VAR-### → value}` (values per Docs 2A/2B/2C)

### **4.5 Expected / Acceptance blocks**

* **Expected (typical):**

  * gate outcomes (`quorum/majority/double_majority/symmetry`)

  * national support %, seat allocations by option, executive winner and IRV summary, frontier statuses per Unit

  * final **label** (`Decisive|Marginal|Invalid`) and reason strings where relevant

* **Acceptance (determinism/perf parts):** flags for identical hashes across runs/OS and performance-within-profile.

---

## **5\) Defaults used in small canonical tests (unless a test overrides)**

* `VM-VAR-001 ballot_type = approval`

* `VM-VAR-010 allocation_method = proportional_favor_small`

* `VM-VAR-012 pr_entry_threshold_pct = 0`

* `VM-VAR-020 quorum_global_pct = 50`

* `VM-VAR-022 national_majority_pct = 55`

* `VM-VAR-023 regional_majority_pct = 55`

* `VM-VAR-024 double_majority_enabled = on`

* `VM-VAR-025 symmetry_enabled = on`

* `VM-VAR-030 weighting_method = population_baseline`

* `VM-VAR-031 aggregate_level = country`

* `VM-VAR-040 frontier_mode = none`

* `VM-VAR-050 tie_policy = status_quo`

* Report precision: one decimal (Doc 7A/7B)

---

## **6\) Notes for implementers**

* Counts in fixtures are authoritative; percentages are derived—do not round twice.

* Use stable ordering (Units by ID; Options by `order_index`) before hashing/serialization to meet Doc 6C determinism tests.

* `expected_canonical_hash` fields are to be filled after the **first certified run** using the canonical serialization rules defined in Doc 3B (sorted keys, LF line endings, UTC timestamps).

---

**Next:** Annex B — Part 1 (Core Allocation Fixtures: VM-TST-001/002/003).

# **Annex B — Part 1: Core Allocation Fixtures (Doc 6A)**

**Covers tests:** VM-TST-001, 002, 003\.  
 **Purpose:** Lock baseline allocation behavior for PR (Sainte-Laguë), WTA, and method convergence on a specific split.  
 **Conventions:** Follow Part 0 (IDs, ordering, rounding, validation).

---

## **VM-TST-001 — Happy PR baseline (Sainte-Laguë)**

**Purpose.** Confirm Sainte-Laguë with `m=10` yields seats **A/B/C/D \= 1/2/3/4**.

**Registry.** Single national unit.

* `REG:CoreAlloc001:1`

* Unit: `U:REG:CoreAlloc001:1:NAT` (level Country, `magnitude=10`, `eligible_roll=100`, `population_baseline=1`, year 2025\)

**Options (order fixed).**

* `OPT:A` (order\_index 1), `OPT:B` (2), `OPT:C` (3), `OPT:D` (4)

**BallotTally (approval; one approval per ballot to satisfy tally-sanity).**

* `TLY:TST001:v1`

* Unit NAT: `ballots_cast=100`, `invalid_or_blank=0`, approvals `{A:10, B:20, C:30, D:40}`

**ParameterSet & expected.**

* `PS:TST001:SainteLague:v1`  
   `VM-VAR-001=approval; VM-VAR-010=proportional_favor_small; VM-VAR-011=on; VM-VAR-012=0; VM-VAR-040=none`

* **Expected seats:** `{A:1, B:2, C:3, D:4}`; **Label:** `Decisive`.

**Canonical fixture (machine-readable).**

{

  "id": "VM-TST-001",

  "registry": {

    "id": "REG:CoreAlloc001:1",

    "provenance": {"source": "AnnexB", "published\_date": "2025-08-11"},

    "units": \[

      {"id": "U:REG:CoreAlloc001:1:NAT","name":"Country","level":"Country","magnitude":10,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025}

    \]

  },

  "options": \[

    {"id":"OPT:A","display\_name":"A","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:B","display\_name":"B","order\_index":2,"is\_status\_quo":false},

    {"id":"OPT:C","display\_name":"C","order\_index":3,"is\_status\_quo":false},

    {"id":"OPT:D","display\_name":"D","order\_index":4,"is\_status\_quo":false}

  \],

  "ballot\_tally": {

    "id":"TLY:TST001:v1",

    "ballot\_type":"approval",

    "units":{

      "U:REG:CoreAlloc001:1:NAT":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:A":10,"OPT:B":20,"OPT:C":30,"OPT:D":40}}

    }

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST001:SainteLague:v1",

      "vars":{"VM-VAR-001":"approval","VM-VAR-010":"proportional\_favor\_small","VM-VAR-011":"on","VM-VAR-012":0,"VM-VAR-040":"none"},

      "expected":{"total\_seats\_by\_party":{"OPT:A":1,"OPT:B":2,"OPT:C":3,"OPT:D":4},"label":"Decisive"}

    }

  \],

  "expected\_canonical\_hash": null

}

---

## **VM-TST-002 — WTA wipe-out**

**Purpose.** Winner-take-all with `m=1` gives full power to the plurality winner (**D**).

**Registry.** Single national unit.

* `REG:CoreAlloc002:1`

* Unit: `U:REG:CoreAlloc002:1:NAT` (`magnitude=1`, `eligible_roll=100`, `population_baseline=1`)

**Options (order fixed).** `OPT:A`..`OPT:D` as above.

**BallotTally (plurality).**

* `TLY:TST002:v1`

* NAT: `ballots_cast=100`, `invalid_or_blank=0`, votes `{A:10, B:20, C:30, D:40}`

**ParameterSet & expected.**

* `PS:TST002:WTA:v1`  
   `VM-VAR-001=plurality; VM-VAR-010=winner_take_all; VM-VAR-011=on; VM-VAR-040=none`

* **Expected power:** `{D:100}` (others 0); **Label:** `Decisive`.

**Canonical fixture.**

{

  "id": "VM-TST-002",

  "registry": {

    "id": "REG:CoreAlloc002:1",

    "provenance": {"source": "AnnexB","published\_date":"2025-08-11"},

    "units": \[

      {"id":"U:REG:CoreAlloc002:1:NAT","name":"Country","level":"Country","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025}

    \]

  },

  "options": \[

    {"id":"OPT:A","display\_name":"A","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:B","display\_name":"B","order\_index":2,"is\_status\_quo":false},

    {"id":"OPT:C","display\_name":"C","order\_index":3,"is\_status\_quo":false},

    {"id":"OPT:D","display\_name":"D","order\_index":4,"is\_status\_quo":false}

  \],

  "ballot\_tally": {

    "id":"TLY:TST002:v1",

    "ballot\_type":"plurality",

    "units":{

      "U:REG:CoreAlloc002:1:NAT":{"ballots\_cast":100,"invalid\_or\_blank":0,"votes":{"OPT:A":10,"OPT:B":20,"OPT:C":30,"OPT:D":40}}

    }

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST002:WTA:v1",

      "vars":{"VM-VAR-001":"plurality","VM-VAR-010":"winner\_take\_all","VM-VAR-011":"on","VM-VAR-040":"none"},

      "expected":{"local\_seats\_by\_party":{"OPT:D":1},"total\_seats\_by\_party":{"OPT:D":1},"label":"Decisive"}

    }

  \],

  "expected\_canonical\_hash": null

}

---

## **VM-TST-003 — Largest Remainder vs Highest-Average (convergent case)**

**Purpose.** With **A/B/C \= 34/33/33** and `m=7`, LR, Sainte-Laguë, and D’Hondt all yield **A/B/C \= 3/2/2**.

**Registry.**

* `REG:CoreAlloc003:1`

* Unit: `U:REG:CoreAlloc003:1:NAT` (`magnitude=7`, `eligible_roll=100`, `population_baseline=1`)

**Options (order fixed).** `OPT:A`, `OPT:B`, `OPT:C`.

**BallotTally (approval; one approval per ballot).**

* `TLY:TST003:v1`

* NAT: `ballots_cast=100`, `invalid_or_blank=0`, approvals `{A:34, B:33, C:33}`

**ParameterSets & expected.**

* `PS:TST003:LR:v1` → `VM-VAR-010=largest_remainder` → seats `{A:3,B:2,C:2}`

* `PS:TST003:SainteLague:v1` → `VM-VAR-010=proportional_favor_small` → `{3,2,2}`

* `PS:TST003:DHondt:v1` → `VM-VAR-010=proportional_favor_big` → `{3,2,2}`

* **Label:** `Decisive` in all three.

**Canonical fixture.**

{

  "id": "VM-TST-003",

  "registry": {

    "id": "REG:CoreAlloc003:1",

    "provenance": {"source":"AnnexB","published\_date":"2025-08-11"},

    "units": \[

      {"id":"U:REG:CoreAlloc003:1:NAT","name":"Country","level":"Country","magnitude":7,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025}

    \]

  },

  "options": \[

    {"id":"OPT:A","display\_name":"A","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:B","display\_name":"B","order\_index":2,"is\_status\_quo":false},

    {"id":"OPT:C","display\_name":"C","order\_index":3,"is\_status\_quo":false}

  \],

  "ballot\_tally": {

    "id":"TLY:TST003:v1",

    "ballot\_type":"approval",

    "units":{

      "U:REG:CoreAlloc003:1:NAT":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:A":34,"OPT:B":33,"OPT:C":33}}

    }

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST003:LR:v1",

      "vars":{"VM-VAR-001":"approval","VM-VAR-010":"largest\_remainder","VM-VAR-011":"on","VM-VAR-012":0},

      "expected":{"total\_seats\_by\_party":{"OPT:A":3,"OPT:B":2,"OPT:C":2},"label":"Decisive"}

    },

    {

      "id":"PS:TST003:SainteLague:v1",

      "vars":{"VM-VAR-001":"approval","VM-VAR-010":"proportional\_favor\_small","VM-VAR-011":"on","VM-VAR-012":0},

      "expected":{"total\_seats\_by\_party":{"OPT:A":3,"OPT:B":2,"OPT:C":2},"label":"Decisive"}

    },

    {

      "id":"PS:TST003:DHondt:v1",

      "vars":{"VM-VAR-001":"approval","VM-VAR-010":"proportional\_favor\_big","VM-VAR-011":"on","VM-VAR-012":0},

      "expected":{"total\_seats\_by\_party":{"OPT:A":3,"OPT:B":2,"OPT:C":2},"label":"Decisive"}

    }

  \],

  "expected\_canonical\_hash": null

}

**Notes (all three tests).**

* Deterministic order **A \> B \> C \> D** via `Option.order_index`.

* No frontier; no gate failures are expected in these fixtures.

* Approval tallies are constructed so `Σ approvals = ballots_cast` to satisfy the simple tally-sanity rule (one approval per ballot in these tests).

**Next:** Annex B — Part 2 (Gates Fixtures: VM-TST-004/005/006/007).

# **Annex B — Part 2: Gates Fixtures (Doc 6B — Quorum/Majority/Double/Symmetry)**

**Covers tests:** VM-TST-004, 005, 006, 007\.  
 **Purpose:** Exercise legitimacy gates using the fixed denominators and rules:

* Approval gate uses **approval rate \= approvals\_for\_change / valid ballots**.

* Quorum uses **eligible\_roll**.

* Double-majority uses national \+ **affected-region family**.

* Symmetry applies identical thresholds in mirrored scenarios.  
   **Conventions:** Follow Part 0 (IDs, ordering, rounding, validation).

---

## **VM-TST-004 — Exact supermajority edge (≥ rule)**

**Purpose.** Show that **55.0%** meets a **55%** supermajority threshold.

**Registry.** One national unit.

* `REG:Gates004:1`

* Unit: `U:REG:Gates004:1:NAT` (Country, `magnitude=1`, `eligible_roll=100`, `population_baseline=1`, year 2025\)

**Options.**

* `OPT:Change` (order\_index 1, `is_status_quo=false`)

* `OPT:StatusQuo` (order\_index 2, `is_status_quo=true`)

**BallotTally (approval).**

* `TLY:TST004:v1`

* NAT: `ballots_cast=100`, `invalid_or_blank=0`, approvals `{Change:55, StatusQuo:45}`

**ParameterSet & expected.**

* `PS:TST004:Edge55:v1`  
   `VM-VAR-001=approval; VM-VAR-020=0; VM-VAR-022=55; VM-VAR-024=on; VM-VAR-040=none`

* **Expected:** Majority **Pass**; **Label:** `Decisive`.

**Canonical fixture.**

{

  "id": "VM-TST-004",

  "registry": {

    "id": "REG:Gates004:1",

    "provenance": {"source":"AnnexB","published\_date":"2025-08-11"},

    "units": \[

      {"id":"U:REG:Gates004:1:NAT","name":"Country","level":"Country","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025}

    \]

  },

  "options": \[

    {"id":"OPT:Change","display\_name":"Change","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:StatusQuo","display\_name":"Status Quo","order\_index":2,"is\_status\_quo":true}

  \],

  "ballot\_tally": {

    "id":"TLY:TST004:v1",

    "ballot\_type":"approval",

    "units":{

      "U:REG:Gates004:1:NAT":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":55,"OPT:StatusQuo":45}}

    }

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST004:Edge55:v1",

      "vars":{"VM-VAR-001":"approval","VM-VAR-020":0,"VM-VAR-022":55,"VM-VAR-024":"on","VM-VAR-040":"none"},

      "expected":{"gates":{"majority":"Pass"},"label":"Decisive"}

    }

  \],

  "expected\_canonical\_hash": null

}

---

## **VM-TST-005 — Quorum failure**

**Purpose.** Turnout **below 50%** invalidates the run even if support ≥ threshold.

**Registry.** One national unit.

* `REG:Gates005:1`

* Unit: `U:REG:Gates005:1:NAT` (`magnitude=1`, `eligible_roll=1000`, `population_baseline=1`, year 2025\)

**Options.** `OPT:Change` (1), `OPT:StatusQuo` (2).

**BallotTally (approval).**

* `TLY:TST005:v1`

* NAT: `ballots_cast=480`, `invalid_or_blank=0`, approvals `{Change:288, StatusQuo:192}` → approval rate for Change \= **60.0%**.

**ParameterSet & expected.**

* `PS:TST005:QuorumFail:v1`  
   `VM-VAR-001=approval; VM-VAR-020=50; VM-VAR-022=55; VM-VAR-024=on; VM-VAR-040=none`

* **Expected:** Quorum **Fail** (turnout 48.0% vs 50%); **Label:** `Invalid` (reason Quorum failed).

**Canonical fixture.**

{

  "id": "VM-TST-005",

  "registry": {

    "id": "REG:Gates005:1",

    "provenance": {"source":"AnnexB","published\_date":"2025-08-11"},

    "units": \[

      {"id":"U:REG:Gates005:1:NAT","name":"Country","level":"Country","magnitude":1,"eligible\_roll":1000,"population\_baseline":1,"population\_baseline\_year":2025}

    \]

  },

  "options": \[

    {"id":"OPT:Change","display\_name":"Change","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:StatusQuo","display\_name":"Status Quo","order\_index":2,"is\_status\_quo":true}

  \],

  "ballot\_tally": {

    "id":"TLY:TST005:v1",

    "ballot\_type":"approval",

    "units":{

      "U:REG:Gates005:1:NAT":{"ballots\_cast":480,"invalid\_or\_blank":0,"approvals":{"OPT:Change":288,"OPT:StatusQuo":192}}

    }

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST005:QuorumFail:v1",

      "vars":{"VM-VAR-001":"approval","VM-VAR-020":50,"VM-VAR-022":55,"VM-VAR-024":"on","VM-VAR-040":"none"},

      "expected":{"gates":{"quorum":"Fail","majority":"Pass"},"label":"Invalid","invalid\_reason":"Quorum failed"}

    }

  \],

  "expected\_canonical\_hash": null

}

---

## **VM-TST-006 — Double-majority failure (affected family by list)**

**Purpose.** National **passes** (57%), but **affected regions** minimum is **53%** → **Fail**.

Note: To satisfy validation (“double-majority with frontier=none must use by\_list/by\_tag”), we define the family **by\_list**.

**Registry.** Three regions.

* `REG:Gates006:1`

* Units (level Region; equal baselines/rolls):

  * `U:REG:Gates006:1:R1` (eligible\_roll=100, pop=1)

  * `U:REG:Gates006:1:R2` (eligible\_roll=100, pop=1)

  * `U:REG:Gates006:1:R3` (eligible\_roll=100, pop=1)

**Options.** `OPT:Change` (1), `OPT:StatusQuo` (2).

**BallotTally (approval).**

* `TLY:TST006:v1`

* R1 approvals `{Change:60, SQ:40}`

* R2 approvals `{Change:58, SQ:42}`

* R3 approvals `{Change:53, SQ:47}`  
   → National approval rate \= (60+58+53)/300 \= **57.0%**.

**ParameterSet & expected.**

* `PS:TST006:DMFail:v1`  
   `VM-VAR-001=approval; VM-VAR-020=0; VM-VAR-022=55; VM-VAR-023=55; VM-VAR-024=on; VM-VAR-026=by_list; VM-VAR-027=["U:REG:Gates006:1:R1","U:REG:Gates006:1:R2","U:REG:Gates006:1:R3"]; VM-VAR-040=none`

* **Expected:** National **Pass**, Double-majority **Fail** (lowest region 53%); **Label:** `Invalid`.

**Canonical fixture.**

{

  "id": "VM-TST-006",

  "registry": {

    "id": "REG:Gates006:1",

    "provenance": {"source":"AnnexB","published\_date":"2025-08-11"},

    "units": \[

      {"id":"U:REG:Gates006:1:R1","name":"R1","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:Gates006:1:R2","name":"R2","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:Gates006:1:R3","name":"R3","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025}

    \]

  },

  "options": \[

    {"id":"OPT:Change","display\_name":"Change","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:StatusQuo","display\_name":"Status Quo","order\_index":2,"is\_status\_quo":true}

  \],

  "ballot\_tally": {

    "id":"TLY:TST006:v1",

    "ballot\_type":"approval",

    "units":{

      "U:REG:Gates006:1:R1":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":60,"OPT:StatusQuo":40}},

      "U:REG:Gates006:1:R2":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":58,"OPT:StatusQuo":42}},

      "U:REG:Gates006:1:R3":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":53,"OPT:StatusQuo":47}}

    }

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST006:DMFail:v1",

      "vars":{"VM-VAR-001":"approval","VM-VAR-020":0,"VM-VAR-022":55,"VM-VAR-023":55,"VM-VAR-024":"on","VM-VAR-026":"by\_list","VM-VAR-027":\["U:REG:Gates006:1:R1","U:REG:Gates006:1:R2","U:REG:Gates006:1:R3"\],"VM-VAR-040":"none"},

      "expected":{"gates":{"majority":"Pass","double\_majority":"Fail"},"label":"Invalid","invalid\_reason":"Regional threshold not met (min 53.0%)"}

    }

  \],

  "expected\_canonical\_hash": null

}

---

## **VM-TST-007 — Symmetry respected (mirrored scenarios)**

**Purpose.** The same thresholds/denominators produce matching Pass results in **A→B** and **B→A** setups with **56%** support.

Implementation note: encoded as **two small subtests** (A and B) to keep Option metadata (`is_status_quo`) consistent without redefining shapes. Both together satisfy Doc 6B’s VM-TST-007.

### **VM-TST-007-A — A→B (Change \= B)**

**Registry.** `REG:Symm007:1`, Unit `...:NAT` (`eligible_roll=100`, baseline=1).

**Options.**

* `OPT:A` (order\_index 1, `is_status_quo=true`)

* `OPT:B` (order\_index 2, `is_status_quo=false`) ← treated as **Change**

**BallotTally.**

* `TLY:TST007A:v1` — approvals `{B:56, A:44}` (valid=100)

**ParameterSet & expected.**

* `PS:TST007A:v1` — `VM-VAR-001=approval; VM-VAR-022=55; VM-VAR-040=none`

* **Expected:** Majority **Pass**; **Label:** `Decisive`.

**Canonical fixture.**

{

  "id": "VM-TST-007-A",

  "registry": {

    "id": "REG:Symm007:1",

    "provenance": {"source":"AnnexB","published\_date":"2025-08-11"},

    "units": \[

      {"id":"U:REG:Symm007:1:NAT","name":"Country","level":"Country","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025}

    \]

  },

  "options": \[

    {"id":"OPT:A","display\_name":"A","order\_index":1,"is\_status\_quo":true},

    {"id":"OPT:B","display\_name":"B","order\_index":2,"is\_status\_quo":false}

  \],

  "ballot\_tally": {

    "id":"TLY:TST007A:v1",

    "ballot\_type":"approval",

    "units":{

      "U:REG:Symm007:1:NAT":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:B":56,"OPT:A":44}}

    }

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST007A:v1",

      "vars":{"VM-VAR-001":"approval","VM-VAR-022":55,"VM-VAR-040":"none"},

      "expected":{"gates":{"majority":"Pass"},"label":"Decisive"}

    }

  \],

  "expected\_canonical\_hash": null

}

### **VM-TST-007-B — B→A (Change \= A)**

**Registry.** Reuse `REG:Symm007:1` (or duplicate if isolation preferred).

**Options.**

* `OPT:A` (order\_index 1, `is_status_quo=false`) ← **Change**

* `OPT:B` (order\_index 2, `is_status_quo=true`)

**BallotTally.**

* `TLY:TST007B:v1` — approvals `{A:56, B:44}`

**ParameterSet & expected.**

* `PS:TST007B:v1` — `VM-VAR-001=approval; VM-VAR-022=55; VM-VAR-040=none`

* **Expected:** Majority **Pass**; **Label:** `Decisive`.

**Canonical fixture.**

{

  "id": "VM-TST-007-B",

  "registry": {

    "id": "REG:Symm007:1",

    "provenance": {"source":"AnnexB","published\_date":"2025-08-11"},

    "units": \[

      {"id":"U:REG:Symm007:1:NAT","name":"Country","level":"Country","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025}

    \]

  },

  "options": \[

    {"id":"OPT:A","display\_name":"A","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:B","display\_name":"B","order\_index":2,"is\_status\_quo":true}

  \],

  "ballot\_tally": {

    "id":"TLY:TST007B:v1",

    "ballot\_type":"approval",

    "units":{

      "U:REG:Symm007:1:NAT":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:A":56,"OPT:B":44}}

    }

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST007B:v1",

      "vars":{"VM-VAR-001":"approval","VM-VAR-022":55,"VM-VAR-040":"none"},

      "expected":{"gates":{"majority":"Pass"},"label":"Decisive"}

    }

  \],

  "expected\_canonical\_hash": null

}

**Notes (Part 2).**

* All approval gate calculations use the **approval rate** denominator (valid ballots).

* Quorum (test 005\) uses `Σ ballots_cast / Σ eligible_roll`.

* Double-majority (test 006\) uses **by\_list** to satisfy validation when `frontier_mode=none`.

* Symmetry (test 007\) is demonstrated via two mirrored subtests with identical thresholds and opposite status-quo designation.

**Next:** Annex B — Part 3 (Ranked Methods Fixtures: VM-TST-010/011).

# **Annex B — Part 3: Ranked Methods Fixtures (Doc 6B — IRV & Condorcet)**

**Covers tests:** VM-TST-010, VM-TST-011.  
 **Purpose:** Exercise ranked-tabulation behaviors: **IRV with exhaustion** and **Condorcet cycle resolved by Schulze**.  
 **Conventions:** Use Part 0 (IDs, ordering, rounding, validation). IRV uses the fixed exhaustion policy **reduce\_continuing\_denominator**.

---

## **VM-TST-010 — IRV with exhaustion**

**Purpose.** Verify IRV round flow, transfers, and exhaustion handling.

**Registry.** Single national unit.

* `REG:Ranked010:1`

* Unit: `U:REG:Ranked010:1:NAT` (Country, `magnitude=1`, `eligible_roll=100`, `population_baseline=1`, year 2025\)

**Options (order fixed).**

* `OPT:A` (order\_index 1), `OPT:B` (2), `OPT:C` (3)

**BallotTally (ranked\_irv).** 100 ballots represented as four groups:

* 40 × `B > A > C`

* 35 × `A > C` *(truncated after 2nd preference)*

* 15 × `C > B`

* 10 × `C` *(no further preferences; will exhaust if C eliminated)*

**ParameterSet & expected.**

* `PS:TST010:IRV:v1` — `VM-VAR-001=ranked_irv; VM-VAR-006=reduce_continuing_denominator`

* **Expected:** R1 A=35, B=40, C=25 → eliminate **C**; transfer 15 to **B**, 10 **exhaust**; continuing=90; final **B=55**, **A=35** ⇒ winner **B**; **Label:** `Decisive`.

**Canonical fixture.**

{

  "id": "VM-TST-010",

  "registry": {

    "id": "REG:Ranked010:1",

    "provenance": {"source":"AnnexB","published\_date":"2025-08-11"},

    "units": \[

      {"id":"U:REG:Ranked010:1:NAT","name":"Country","level":"Country","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025}

    \]

  },

  "options": \[

    {"id":"OPT:A","display\_name":"A","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:B","display\_name":"B","order\_index":2,"is\_status\_quo":false},

    {"id":"OPT:C","display\_name":"C","order\_index":3,"is\_status\_quo":false}

  \],

  "ballot\_tally": {

    "id":"TLY:TST010:v1",

    "ballot\_type":"ranked\_irv",

    "unit":"U:REG:Ranked010:1:NAT",

    "rounds":\[

      {"ranking":\["OPT:B","OPT:A","OPT:C"\],"count":40},

      {"ranking":\["OPT:A","OPT:C"\],"count":35},

      {"ranking":\["OPT:C","OPT:B"\],"count":15},

      {"ranking":\["OPT:C"\],"count":10}

    \],

    "exhaustion\_policy":"reduce\_continuing\_denominator"

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST010:IRV:v1",

      "vars":{"VM-VAR-001":"ranked\_irv","VM-VAR-006":"reduce\_continuing\_denominator"},

      "expected":{

        "executive\_winner":"OPT:B",

        "executive\_irv\_summary":{

          "exhausted\_ballots":10,

          "final\_continuing":90,

          "final\_round":{"OPT:B":55,"OPT:A":35}

        },

        "label":"Decisive"

      }

    }

  \],

  "expected\_canonical\_hash": null

}

---

## **VM-TST-011 — Condorcet cycle resolved (Schulze)**

**Purpose.** Create a **rock–paper–scissors** cycle (A\>B, B\>C, C\>A) and confirm the **Schulze winner \= B**.

**Registry.** Single national unit.

* `REG:Ranked011:1`

* Unit: `U:REG:Ranked011:1:NAT` (`magnitude=1`, `eligible_roll=100`, baseline=1, year 2025\)

**Options (order fixed).**

* `OPT:A` (1), `OPT:B` (2), `OPT:C` (3)

**BallotTally (ranked\_condorcet).** 100 ballots across **all six permutations** to produce a cycle where Schulze selects **B**:

* 25 × `A > B > C` → **a**

* 10 × `A > C > B` → **b**

* 5 × `B > A > C` → **c**

* 30 × `B > C > A` → **d**

* 20 × `C > A > B` → **e**

* 10 × `C > B > A` → **f**

(This profile yields head-to-heads: **A\>B 55–45**, **B\>C 60–40**, **C\>A 60–40**; strongest paths favor **B** in the Schulze relation.)

**ParameterSet & expected.**

* `PS:TST011:Schulze:v1` — `VM-VAR-001=ranked_condorcet; VM-VAR-005=schulze`

* **Expected:** **Winner \= B**, **Label:** `Decisive`.

**Canonical fixture.**

{

  "id": "VM-TST-011",

  "registry": {

    "id": "REG:Ranked011:1",

    "provenance": {"source":"AnnexB","published\_date":"2025-08-11"},

    "units": \[

      {"id":"U:REG:Ranked011:1:NAT","name":"Country","level":"Country","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025}

    \]

  },

  "options": \[

    {"id":"OPT:A","display\_name":"A","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:B","display\_name":"B","order\_index":2,"is\_status\_quo":false},

    {"id":"OPT:C","display\_name":"C","order\_index":3,"is\_status\_quo":false}

  \],

  "ballot\_tally": {

    "id":"TLY:TST011:v1",

    "ballot\_type":"ranked\_condorcet",

    "ballots":\[

      {"ranking":\["OPT:A","OPT:B","OPT:C"\],"count":25},

      {"ranking":\["OPT:A","OPT:C","OPT:B"\],"count":10},

      {"ranking":\["OPT:B","OPT:A","OPT:C"\],"count":5},

      {"ranking":\["OPT:B","OPT:C","OPT:A"\],"count":30},

      {"ranking":\["OPT:C","OPT:A","OPT:B"\],"count":20},

      {"ranking":\["OPT:C","OPT:B","OPT:A"\],"count":10}

    \],

    "completion":"schulze"

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST011:Schulze:v1",

      "vars":{"VM-VAR-001":"ranked\_condorcet","VM-VAR-005":"schulze"},

      "expected":{"executive\_winner":"OPT:B","label":"Decisive"}

    }

  \],

  "expected\_canonical\_hash": null

}

**Notes (Part 3).**

* The **IRV** fixture guarantees **ballot exhaustion**, so engines must show reduced continuing denominators in the final round (90).

* The **Condorcet** fixture encodes a concrete ballot profile that realizes the illustrative pairwise margins from Doc 6B; engines must compute the **Schulze** paths to select **B**.

**Next:** Annex B — Part 4 (Weighting & MMP Level Fixtures: VM-TST-012/013).

# **Annex B — Part 4: Weighting & MMP Level Fixtures (Doc 6B)**

**Covers tests:** VM-TST-012, VM-TST-013.  
 **Purpose:** Exercise (1) national support flipping under different weighting methods and (2) **MMP** seat totals changing with `mlc_correction_level`.  
 **Conventions:** Use Part 0 (IDs, ordering, rounding, validation).

---

## **VM-TST-012 — Weighting flip (equal-unit vs population)**

**Purpose.** Show national support changes from **Pass (60.0%)** to **Fail (46.7%)** when switching weighting from `equal_unit` to `population_baseline`.

**Registry.** Four Units (two small, two large).

* `REG:Weighting012:1`

* Units (Country level; each `magnitude=1`):

  * `U:REG:Weighting012:1:S1` Small1 — `eligible_roll=100`, `population_baseline=1`

  * `U:REG:Weighting012:1:S2` Small2 — `eligible_roll=100`, `population_baseline=1`

  * `U:REG:Weighting012:1:L1` Large1 — `eligible_roll=1000`, `population_baseline=10`

  * `U:REG:Weighting012:1:L2` Large2 — `eligible_roll=1000`, `population_baseline=10`

**Options.**

* `OPT:Change` (order\_index 1\)

* `OPT:StatusQuo` (order\_index 2, `is_status_quo=true`)

**BallotTally (approval).**

* Small1: Change **80**, SQ **20**

* Small2: Change **80**, SQ **20**

* Large1: Change **400**, SQ **600**

* Large2: Change **400**, SQ **600**  
   (Valid \= ballots\_cast in each unit; no blanks.)

**ParameterSets & expected.**

* **Case 1 (equal-unit):** `VM-VAR-030=equal_unit` ⇒ national Change \= (80+80+40+40)/4 \= **60.0%** ⇒ **Majority Pass**, **Label Decisive**.

* **Case 2 (population):** `VM-VAR-030=population_baseline` ⇒ weighted Change \= (80·1 \+ 80·1 \+ 40·10 \+ 40·10) / (1+1+10+10) \= **46.7%** ⇒ **Majority Fail**, **Label Invalid**.

**Canonical fixture.**

{

  "id": "VM-TST-012",

  "registry": {

    "id": "REG:Weighting012:1",

    "provenance": {"source": "AnnexB", "published\_date": "2025-08-11"},

    "units": \[

      {"id":"U:REG:Weighting012:1:S1","name":"Small1","level":"Country","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:Weighting012:1:S2","name":"Small2","level":"Country","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:Weighting012:1:L1","name":"Large1","level":"Country","magnitude":1,"eligible\_roll":1000,"population\_baseline":10,"population\_baseline\_year":2025},

      {"id":"U:REG:Weighting012:1:L2","name":"Large2","level":"Country","magnitude":1,"eligible\_roll":1000,"population\_baseline":10,"population\_baseline\_year":2025}

    \]

  },

  "options": \[

    {"id":"OPT:Change","display\_name":"Change","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:StatusQuo","display\_name":"Status Quo","order\_index":2,"is\_status\_quo":true}

  \],

  "ballot\_tally": {

    "id":"TLY:TST012:v1",

    "ballot\_type":"approval",

    "units":{

      "U:REG:Weighting012:1:S1":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":80,"OPT:StatusQuo":20}},

      "U:REG:Weighting012:1:S2":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":80,"OPT:StatusQuo":20}},

      "U:REG:Weighting012:1:L1":{"ballots\_cast":1000,"invalid\_or\_blank":0,"approvals":{"OPT:Change":400,"OPT:StatusQuo":600}},

      "U:REG:Weighting012:1:L2":{"ballots\_cast":1000,"invalid\_or\_blank":0,"approvals":{"OPT:Change":400,"OPT:StatusQuo":600}}

    }

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST012:EqualUnit:v1",

      "vars":{"VM-VAR-001":"approval","VM-VAR-010":"proportional\_favor\_small","VM-VAR-012":0,"VM-VAR-020":0,"VM-VAR-022":55,"VM-VAR-024":"on","VM-VAR-025":"on","VM-VAR-030":"equal\_unit","VM-VAR-031":"country","VM-VAR-040":"none"},

      "expected":{"national\_support\_pct":60.0,"gates":{"majority":"Pass"},"label":"Decisive"}

    },

    {

      "id":"PS:TST012:PopWeighted:v1",

      "vars":{"VM-VAR-001":"approval","VM-VAR-010":"proportional\_favor\_small","VM-VAR-012":0,"VM-VAR-020":0,"VM-VAR-022":55,"VM-VAR-024":"on","VM-VAR-025":"on","VM-VAR-030":"population\_baseline","VM-VAR-031":"country","VM-VAR-040":"none"},

      "expected":{"national\_support\_pct":46.7,"gates":{"majority":"Fail"},"label":"Invalid","invalid\_reason":"Majority threshold not met"}

    }

  \],

  "expected\_canonical\_hash": null

}

---

## **VM-TST-013 — MMP correction level (national vs regional)**

**Purpose.** Prove `mlc_correction_level` changes final seat totals under **MMP**.

**Setup summary.** Three equal-population regions; **12 total seats**: 6 local (SMD WTA) \+ 6 top-up (50%). Local winners:

* Region1: **A, A** (2 SMDs)

* Region2: **B, B**

* Region3: **C, C**  
   Regional vote shares (for top-up target seats):

* R1: A **90%**, B 5%, C 5%

* R2: B **55%**, A 40%, C 5%

* R3: C **55%**, A 40%, B 5%  
   Implied **national shares** ≈ A **56.7%**, B **21.7%**, C **21.7%**.

**Expected outcomes.**

* **Case 1 (national correction):** totals **A/B/C \= 7/3/2**.

* **Case 2 (regional correction):** totals **A/B/C \= 8/2/2**.  
   Both **Decisive**.

**Registry.** Regions with two SMDs each (SMDs have `magnitude=1`; parent regions hold `magnitude=0` for clarity).

* `REG:MMP013:1`

* Units: `R1` with `R1:S1`, `R1:S2`; `R2` with `R2:S1`, `R2:S2`; `R3` with `R3:S1`, `R3:S2`.

**Options.** `OPT:A` (1), `OPT:B` (2), `OPT:C` (3).

**BallotTally (approval used to fix local winners and compute shares).**

* R1 SMDs: A 270, B 15, C 15 (both SMDs identical)

* R2 SMDs: B 165, A 120, C 15 (both SMDs identical)

* R3 SMDs: C 165, A 120, B 15 (both SMDs identical)

**ParameterSets & expected.**

* **National correction:** `VM-VAR-016=national` ⇒ final seats **A7/B3/C2**.

* **Regional correction:** `VM-VAR-016=regional` ⇒ per-region corrections yield **A8/B2/C2**.

**Canonical fixture.**

{

  "id": "VM-TST-013",

  "registry": {

    "id": "REG:MMP013:1",

    "provenance": {"source": "AnnexB", "published\_date": "2025-08-11"},

    "units": \[

      {"id":"U:REG:MMP013:1:R1","name":"Region1","level":"Region","magnitude":0,"eligible\_roll":600,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:MMP013:1:R1:S1","name":"R1-SMD1","level":"District","parent":"U:REG:MMP013:1:R1","magnitude":1,"eligible\_roll":300,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:MMP013:1:R1:S2","name":"R1-SMD2","level":"District","parent":"U:REG:MMP013:1:R1","magnitude":1,"eligible\_roll":300,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:MMP013:1:R2","name":"Region2","level":"Region","magnitude":0,"eligible\_roll":600,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:MMP013:1:R2:S1","name":"R2-SMD1","level":"District","parent":"U:REG:MMP013:1:R2","magnitude":1,"eligible\_roll":300,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:MMP013:1:R2:S2","name":"R2-SMD2","level":"District","parent":"U:REG:MMP013:1:R2","magnitude":1,"eligible\_roll":300,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:MMP013:1:R3","name":"Region3","level":"Region","magnitude":0,"eligible\_roll":600,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:MMP013:1:R3:S1","name":"R3-SMD1","level":"District","parent":"U:REG:MMP013:1:R3","magnitude":1,"eligible\_roll":300,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:MMP013:1:R3:S2","name":"R3-SMD2","level":"District","parent":"U:REG:MMP013:1:R3","magnitude":1,"eligible\_roll":300,"population\_baseline":1,"population\_baseline\_year":2025}

    \]

  },

  "options": \[

    {"id":"OPT:A","display\_name":"A","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:B","display\_name":"B","order\_index":2,"is\_status\_quo":false},

    {"id":"OPT:C","display\_name":"C","order\_index":3,"is\_status\_quo":false}

  \],

  "ballot\_tally": {

    "id":"TLY:TST013:v1",

    "ballot\_type":"approval",

    "units":{

      "U:REG:MMP013:1:R1:S1":{"ballots\_cast":300,"invalid\_or\_blank":0,"approvals":{"OPT:A":270,"OPT:B":15,"OPT:C":15}},

      "U:REG:MMP013:1:R1:S2":{"ballots\_cast":300,"invalid\_or\_blank":0,"approvals":{"OPT:A":270,"OPT:B":15,"OPT:C":15}},

      "U:REG:MMP013:1:R2:S1":{"ballots\_cast":300,"invalid\_or\_blank":0,"approvals":{"OPT:B":165,"OPT:A":120,"OPT:C":15}},

      "U:REG:MMP013:1:R2:S2":{"ballots\_cast":300,"invalid\_or\_blank":0,"approvals":{"OPT:B":165,"OPT:A":120,"OPT:C":15}},

      "U:REG:MMP013:1:R3:S1":{"ballots\_cast":300,"invalid\_or\_blank":0,"approvals":{"OPT:C":165,"OPT:A":120,"OPT:B":15}},

      "U:REG:MMP013:1:R3:S2":{"ballots\_cast":300,"invalid\_or\_blank":0,"approvals":{"OPT:C":165,"OPT:A":120,"OPT:B":15}}

    }

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST013:National:v1",

      "vars":{"VM-VAR-001":"approval","VM-VAR-010":"mixed\_local\_correction","VM-VAR-011":"on","VM-VAR-012":0,"VM-VAR-013":50,"VM-VAR-014":"allow\_overhang","VM-VAR-015":"natural\_vote\_share","VM-VAR-016":"national","VM-VAR-017":"fixed\_total","VM-VAR-030":"population\_baseline","VM-VAR-031":"country","VM-VAR-040":"none"},

      "expected":{"local\_seats\_by\_party":{"OPT:A":2,"OPT:B":2,"OPT:C":2},"total\_seats\_by\_party":{"OPT:A":7,"OPT:B":3,"OPT:C":2},"label":"Decisive"}

    },

    {

      "id":"PS:TST013:Regional:v1",

      "vars":{"VM-VAR-001":"approval","VM-VAR-010":"mixed\_local\_correction","VM-VAR-011":"on","VM-VAR-012":0,"VM-VAR-013":50,"VM-VAR-014":"allow\_overhang","VM-VAR-015":"natural\_vote\_share","VM-VAR-016":"regional","VM-VAR-017":"fixed\_total","VM-VAR-030":"population\_baseline","VM-VAR-031":"country","VM-VAR-040":"none"},

      "expected":{"local\_seats\_by\_party":{"OPT:A":2,"OPT:B":2,"OPT:C":2},"total\_seats\_by\_party":{"OPT:A":8,"OPT:B":2,"OPT:C":2},"label":"Decisive"}

    }

  \],

  "expected\_canonical\_hash": null

}

**Notes (Part 4).**

* **VM-TST-012** uses approval ballots and treats unit-level support as the basis for national aggregation under different weightings; gate checks use the **approval rate** denominator.

* **VM-TST-013** encodes local winners via strong per-SMD tallies; top-up math follows Doc 4B with `mlc_topup_share_pct=50`, `total_seats_model=fixed_total`, and deficit-driven assignment.

* Deterministic option order is **A \> B \> C** (where relevant).

**Next:** Annex B — Part 5 (Frontier Mapping Fixtures: VM-TST-014/015/016/017).

# **Annex B — Part 5: Frontier Mapping Fixtures (Doc 6C)**

**Covers tests:** VM-TST-014, 015, 016, 017\.  
 **Purpose:** Exercise frontier mapping across binary/sliding modes, contiguity policies, and protected areas.  
 **Conventions:** Use Part 0 (IDs, ordering, rounding, validation). Labels follow Doc 4C (mediation/protected flags ⇒ **Marginal**).

---

## **VM-TST-014 — Binary cutoff with a contiguity break**

**Purpose.** Require support ≥ cutoff **and** contiguity under allowed modes.

**Canonical fixture**

{

  "id": "VM-TST-014",

  "registry": {

    "id": "REG:FrontierFive:1",

    "provenance": {"source": "AnnexB", "published\_date": "2025-08-11"},

    "units": \[

      {"id":"U:REG:FrontierFive:1:U1","name":"U1","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:FrontierFive:1:U2","name":"U2","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:FrontierFive:1:U3","name":"U3","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:FrontierFive:1:U4","name":"U4","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:FrontierFive:1:U5","name":"U5","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025}

    \],

    "adjacency": \[

      {"a":"U:REG:FrontierFive:1:U1","b":"U:REG:FrontierFive:1:U2","type":"land"},

      {"a":"U:REG:FrontierFive:1:U2","b":"U:REG:FrontierFive:1:U3","type":"land"},

      {"a":"U:REG:FrontierFive:1:U3","b":"U:REG:FrontierFive:1:U5","type":"land"},

      {"a":"U:REG:FrontierFive:1:U4","b":"U:REG:FrontierFive:1:U3","type":"water"}

    \]

  },

  "options": \[

    {"id":"OPT:Change","display\_name":"Change","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:StatusQuo","display\_name":"Status Quo","order\_index":2,"is\_status\_quo":true}

  \],

  "ballot\_tally": {

    "id":"TLY:TST014:v1",

    "ballot\_type":"approval",

    "units":{

      "U:REG:FrontierFive:1:U1":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":62,"OPT:StatusQuo":38}},

      "U:REG:FrontierFive:1:U2":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":61,"OPT:StatusQuo":39}},

      "U:REG:FrontierFive:1:U3":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":45,"OPT:StatusQuo":55}},

      "U:REG:FrontierFive:1:U4":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":65,"OPT:StatusQuo":35}},

      "U:REG:FrontierFive:1:U5":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":30,"OPT:StatusQuo":70}}

    }

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST014:Frontier:v1",

      "vars":{

        "VM-VAR-001":"approval",

        "VM-VAR-040":"binary\_cutoff",

        "VM-VAR-041":60,

        "VM-VAR-047":\["land"\],

        "VM-VAR-048":"none"

      },

      "expected":{

        "frontier\_status":{

          "U:REG:FrontierFive:1:U1":"immediate\_change",

          "U:REG:FrontierFive:1:U2":"immediate\_change",

          "U:REG:FrontierFive:1:U3":"no\_change",

          "U:REG:FrontierFive:1:U4":"mediation",

          "U:REG:FrontierFive:1:U5":"no\_change"

        },

        "label":"Marginal",

        "marginal\_reason":"Mediation present"

      }

    }

  \],

  "expected\_canonical\_hash": null

}

---

## **VM-TST-015 — Sliding-scale bands with autonomy package**

**Purpose.** Band assignment is single and deterministic; autonomy package mapping applied.

**Canonical fixture**

{

  "id": "VM-TST-015",

  "registry": {

    "id": "REG:FrontierFour:1",

    "provenance": {"source": "AnnexB", "published\_date": "2025-08-11"},

    "units": \[

      {"id":"U:REG:FrontierFour:1:U1","name":"U1","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:FrontierFour:1:U2","name":"U2","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:FrontierFour:1:U3","name":"U3","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:FrontierFour:1:U4","name":"U4","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025}

    \],

    "adjacency": \[

      {"a":"U:REG:FrontierFour:1:U1","b":"U:REG:FrontierFour:1:U2","type":"land"},

      {"a":"U:REG:FrontierFour:1:U2","b":"U:REG:FrontierFour:1:U3","type":"land"},

      {"a":"U:REG:FrontierFour:1:U3","b":"U:REG:FrontierFour:1:U4","type":"land"}

    \]

  },

  "options": \[

    {"id":"OPT:Change","display\_name":"Change","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:StatusQuo","display\_name":"Status Quo","order\_index":2,"is\_status\_quo":true}

  \],

  "ballot\_tally": {

    "id":"TLY:TST015:v1",

    "ballot\_type":"approval",

    "units":{

      "U:REG:FrontierFour:1:U1":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":25,"OPT:StatusQuo":75}},

      "U:REG:FrontierFour:1:U2":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":35,"OPT:StatusQuo":65}},

      "U:REG:FrontierFour:1:U3":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":52,"OPT:StatusQuo":48}},

      "U:REG:FrontierFour:1:U4":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":61,"OPT:StatusQuo":39}}

    }

  },

  "autonomy\_packages": \[

    {"id":"AP:Base:v1","powers":\["language","education"\],"review\_period\_years":5}

  \],

  "parameter\_sets": \[

    {

      "id":"PS:TST015:Sliding:v1",

      "vars":{

        "VM-VAR-001":"approval",

        "VM-VAR-040":"sliding\_scale",

        "VM-VAR-042":\[

          {"min\_pct":0,"max\_pct":29,"action":"no\_change"},

          {"min\_pct":30,"max\_pct":49,"action":"autonomy(AP:Base)"},

          {"min\_pct":50,"max\_pct":59,"action":"phased\_change"},

          {"min\_pct":60,"max\_pct":100,"action":"immediate\_change"}

        \],

        "VM-VAR-046":{"autonomy(AP:Base)":"AP:Base:v1"},

        "VM-VAR-047":\["land","bridge"\],

        "VM-VAR-048":"none"

      },

      "expected":{

        "frontier\_status":{

          "U:REG:FrontierFour:1:U1":"no\_change",

          "U:REG:FrontierFour:1:U2":"autonomy(AP:Base:v1)",

          "U:REG:FrontierFour:1:U3":"phased\_change",

          "U:REG:FrontierFour:1:U4":"immediate\_change"

        },

        "label":"Decisive"

      }

    }

  \],

  "expected\_canonical\_hash": null

}

---

## **VM-TST-016 — Protected area blocks change (no override)**

**Purpose.** Protected units cannot change without explicit override; presence of protected block ⇒ **Marginal**.

**Canonical fixture**

{

  "id": "VM-TST-016",

  "registry": {

    "id": "REG:FrontierProtected:1",

    "provenance": {"source": "AnnexB", "published\_date": "2025-08-11"},

    "units": \[

      {"id":"U:REG:FrontierProtected:1:U1","name":"U1","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025,"protected\_area":true},

      {"id":"U:REG:FrontierProtected:1:U2","name":"U2","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025},

      {"id":"U:REG:FrontierProtected:1:U3","name":"U3","level":"Region","magnitude":1,"eligible\_roll":100,"population\_baseline":1,"population\_baseline\_year":2025}

    \],

    "adjacency": \[

      {"a":"U:REG:FrontierProtected:1:U1","b":"U:REG:FrontierProtected:1:U2","type":"land"},

      {"a":"U:REG:FrontierProtected:1:U2","b":"U:REG:FrontierProtected:1:U3","type":"land"}

    \]

  },

  "options": \[

    {"id":"OPT:Change","display\_name":"Change","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:StatusQuo","display\_name":"Status Quo","order\_index":2,"is\_status\_quo":true}

  \],

  "ballot\_tally": {

    "id":"TLY:TST016:v1",

    "ballot\_type":"approval",

    "units":{

      "U:REG:FrontierProtected:1:U1":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":70,"OPT:StatusQuo":30}},

      "U:REG:FrontierProtected:1:U2":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":62,"OPT:StatusQuo":38}},

      "U:REG:FrontierProtected:1:U3":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":41,"OPT:StatusQuo":59}}

    }

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST016:ProtectedNoOverride:v1",

      "vars":{

        "VM-VAR-001":"approval",

        "VM-VAR-040":"binary\_cutoff",

        "VM-VAR-041":60,

        "VM-VAR-045":"off",

        "VM-VAR-047":\["land"\],

        "VM-VAR-048":"none"

      },

      "expected":{

        "frontier\_status":{

          "U:REG:FrontierProtected:1:U1":"no\_change",

          "U:REG:FrontierProtected:1:U2":"immediate\_change",

          "U:REG:FrontierProtected:1:U3":"no\_change"

        },

        "label":"Marginal",

        "marginal\_reason":"Protected unit blocked change"

      }

    }

  \],

  "expected\_canonical\_hash": null

}

---

## **VM-TST-017 — Diffuse support floor (no change anywhere)**

**Purpose.** All units below the band floor map to **no\_change**; no flags ⇒ **Decisive**.

**Canonical fixture**

{

  "id": "VM-TST-017",

  "registry": {

    "id": "REG:FrontierSix:1",

    "provenance": {"source": "AnnexB", "published\_date": "2025-08-11"},

    "units": \[

      {"id":"U:REG:FrontierSix:1:U1","name":"U1","level":"Region","magnitude":1,"eligible\_roll":100},

      {"id":"U:REG:FrontierSix:1:U2","name":"U2","level":"Region","magnitude":1,"eligible\_roll":100},

      {"id":"U:REG:FrontierSix:1:U3","name":"U3","level":"Region","magnitude":1,"eligible\_roll":100},

      {"id":"U:REG:FrontierSix:1:U4","name":"U4","level":"Region","magnitude":1,"eligible\_roll":100},

      {"id":"U:REG:FrontierSix:1:U5","name":"U5","level":"Region","magnitude":1,"eligible\_roll":100},

      {"id":"U:REG:FrontierSix:1:U6","name":"U6","level":"Region","magnitude":1,"eligible\_roll":100}

    \]

  },

  "options": \[

    {"id":"OPT:Change","display\_name":"Change","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:StatusQuo","display\_name":"Status Quo","order\_index":2,"is\_status\_quo":true}

  \],

  "ballot\_tally": {

    "id":"TLY:TST017:v1",

    "ballot\_type":"approval",

    "units":{

      "U:REG:FrontierSix:1:U1":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":20,"OPT:StatusQuo":80}},

      "U:REG:FrontierSix:1:U2":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":28,"OPT:StatusQuo":72}},

      "U:REG:FrontierSix:1:U3":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":33,"OPT:StatusQuo":67}},

      "U:REG:FrontierSix:1:U4":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":35,"OPT:StatusQuo":65}},

      "U:REG:FrontierSix:1:U5":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":36,"OPT:StatusQuo":64}},

      "U:REG:FrontierSix:1:U6":{"ballots\_cast":100,"invalid\_or\_blank":0,"approvals":{"OPT:Change":39,"OPT:StatusQuo":61}}

    }

  },

  "parameter\_sets": \[

    {

      "id":"PS:TST017:Sliding:v1",

      "vars":{

        "VM-VAR-001":"approval",

        "VM-VAR-040":"sliding\_scale",

        "VM-VAR-042":\[

          {"min\_pct":0,"max\_pct":39,"action":"no\_change"},

          {"min\_pct":40,"max\_pct":59,"action":"phased\_change"},

          {"min\_pct":60,"max\_pct":100,"action":"immediate\_change"}

        \]

      },

      "expected":{

        "frontier\_status":{

          "U:REG:FrontierSix:1:U1":"no\_change",

          "U:REG:FrontierSix:1:U2":"no\_change",

          "U:REG:FrontierSix:1:U3":"no\_change",

          "U:REG:FrontierSix:1:U4":"no\_change",

          "U:REG:FrontierSix:1:U5":"no\_change",

          "U:REG:FrontierSix:1:U6":"no\_change"

        },

        "label":"Decisive"

      }

    }

  \],

  "expected\_canonical\_hash": null

}

**Notes (Part 5).**

* VM-VAR-047/048 enforce contiguity/island policies; mediation arises when a unit meets cutoff/band but lacks required contiguity or is isolated by disallowed modes.

* VM-VAR-045 prevents protected units from changing unless explicitly overridden; any such block triggers **Marginal**.

* All counts satisfy tally sanity; labels depend only on frontier flags here (gates assumed pass).

**Next:** Annex B — Part 6 (Executive \+ Council Fixture: VM-TST-018).

# **Annex B — Part 6: Executive \+ Council Fixtures (Doc 6C)**

**Covers tests:** VM-TST-018  
 **Purpose:** Mixed institutions — **IRV executive** alongside **PR council**.  
 **Conventions:** Use Part 0 (IDs, ordering, rounding, validation). IRV uses `reduce_continuing_denominator`.

---

## **VM-TST-018 — Executive (IRV) \+ Council (PR)**

**Intent.** Confirm IRV winner and Sainte-Laguë council seats computed from the same run context.

**Canonical fixture**

{

  "id": "VM-TST-018",

  "registry": {

    "id": "REG:ExecCouncil:1",

    "provenance": {"source": "AnnexB", "published\_date": "2025-08-11"},

    "units": \[

      {

        "id": "U:REG:ExecCouncil:1:NAT",

        "name": "Country",

        "level": "Country",

        "magnitude": 15,

        "eligible\_roll": 1000,

        "population\_baseline": 1,

        "population\_baseline\_year": 2025

      }

    \]

  },

  "options": \[

    {"id":"OPT:A","display\_name":"A","order\_index":1,"is\_status\_quo":false},

    {"id":"OPT:B","display\_name":"B","order\_index":2,"is\_status\_quo":false},

    {"id":"OPT:C","display\_name":"C","order\_index":3,"is\_status\_quo":false},

    {"id":"OPT:D","display\_name":"D","order\_index":4,"is\_status\_quo":false}

  \],

  "ballot\_tally": {

    "id": "TLY:TST018:v1",

    "executive": {

      "ballot\_type": "ranked\_irv",

      "unit": "U:REG:ExecCouncil:1:NAT",

      "rounds": \[

        {"ranking": \["OPT:B","OPT:A","OPT:C"\], "count": 40},

        {"ranking": \["OPT:A","OPT:C"\], "count": 35},

        {"ranking": \["OPT:C","OPT:B"\], "count": 15},

        {"ranking": \["OPT:C"\], "count": 10}

      \],

      "exhaustion\_policy": "reduce\_continuing\_denominator"

    },

    "council": {

      "ballot\_type": "approval",

      "units": {

        "U:REG:ExecCouncil:1:NAT": {

          "ballots\_cast": 1000,

          "invalid\_or\_blank": 0,

          "approvals": {

            "OPT:D": 400,

            "OPT:C": 300,

            "OPT:B": 200,

            "OPT:A": 100

          }

        }

      }

    }

  },

  "parameter\_sets": \[

    {

      "id": "PS:TST018:ExecIRV+CouncilPR:v1",

      "vars": {

        "VM-VAR-001": "approval",

        "VM-VAR-006": "reduce\_continuing\_denominator",

        "VM-VAR-010": "proportional\_favor\_small",

        "VM-VAR-011": "on",

        "VM-VAR-012": 5,

        "VM-VAR-030": "population\_baseline",

        "VM-VAR-031": "country",

        "VM-VAR-040": "none",

        "VM-VAR-050": "status\_quo"

      },

      "expected": {

        "executive\_winner": "OPT:B",

        "executive\_irv\_summary": {

          "exhausted\_ballots": 10,

          "final\_continuing": 90,

          "final\_round": {"OPT:B": 55, "OPT:A": 35}

        },

        "council\_seats\_by\_party": {"OPT:D": 6, "OPT:C": 5, "OPT:B": 3, "OPT:A": 1},

        "label": "Decisive"

      }

    }

  \],

  "expected\_canonical\_hash": null

}

**Notes (Part 6).**

* Executive and council tallies are provided side-by-side under one `BallotTally` ID to ensure consistent provenance.

* Council seats are computed with **Sainte-Laguë** (Doc 4B) and `magnitude=15` at the national unit; PR threshold \= **5%**.

* IRV summary must show **exhausted=10**, **continuing=90**, final **55–35** split.

* Labels depend on gates/frontier; here none are triggered, so **Decisive**.

**Next:** Annex B — Part 7 (Determinism & Cross-OS Fixtures: VM-TST-019/020).

# **Annex B — Part 7: Determinism & Cross-OS Fixtures (Doc 6C)**

**Covers tests:** VM-TST-019, VM-TST-020  
 **Purpose:** Prove byte-identical outputs on repeat (same OS) and across Windows/macOS/Linux, while staying within published performance profiles.  
 **Conventions:** Use Part 0 (IDs, ordering, rounding, validation). Hashes filled post-certification.

---

## **VM-TST-019 — Determinism & performance (large synthetic, same OS)**

**Intent.** Repeating the same run on the same machine/OS yields identical `Result` and `RunRecord` hashes; runtime/memory within published profile.

{

  "id": "VM-TST-019",

  "title": "Determinism & performance — large synthetic",

  "purpose": "Byte-identical outputs on repeated same-OS runs; within perf/memory gates.",

  "generator": {

    "seed": 20250811,

    "units": 5000,

    "levels": \["Country"\],

    "options": \[

      {"id":"OPT:A","display\_name":"A","order\_index":1,"is\_status\_quo":false},

      {"id":"OPT:B","display\_name":"B","order\_index":2,"is\_status\_quo":false},

      {"id":"OPT:C","display\_name":"C","order\_index":3,"is\_status\_quo":false},

      {"id":"OPT:D","display\_name":"D","order\_index":4,"is\_status\_quo":false}

    \],

    "ballots": {

      "type": "approval",

      "avg\_turnout": 600,

      "invalid\_rate": 0.01

    },

    "weights": {

      "population\_baseline\_range": \[1, 10\],

      "population\_baseline\_year": 2025

    }

  },

  "parameter\_set": {

    "id": "PS:TST019:Baseline:v1",

    "vars": {

      "VM-VAR-001": "approval",

      "VM-VAR-010": "proportional\_favor\_small",

      "VM-VAR-011": "on",

      "VM-VAR-012": 0,

      "VM-VAR-020": 0,

      "VM-VAR-022": 55,

      "VM-VAR-024": "on",

      "VM-VAR-025": "on",

      "VM-VAR-030": "population\_baseline",

      "VM-VAR-031": "country",

      "VM-VAR-040": "none",

      "VM-VAR-050": "status\_quo"

    }

  },

  "acceptance": {

    "repeat\_runs\_same\_os": "identical\_result\_and\_runrecord\_hashes",

    "perf\_within\_profile": true,

    "perf\_profile\_ref": "profiles/engine-vX.Y.Z/\<os-arch\>.json"

  },

  "expected\_canonical\_hash": null

}

**Notes.** Engine must serialize with sorted keys, LF line endings, UTC timestamps; same binary \+ same inputs ⇒ identical hashes.

---

## **VM-TST-020 — Cross-OS determinism (Windows/macOS/Linux)**

**Intent.** Running the same canonical case on Windows, macOS, and Linux produces byte-identical `Result` and `RunRecord`. Uses the small baseline from VM-TST-001.

{

  "id": "VM-TST-020",

  "title": "Cross-OS determinism",

  "purpose": "Byte-identical outputs on Windows, macOS, and Linux.",

  "registry\_ref": "VM-TST-001.registry",

  "ballot\_tally\_ref": "VM-TST-001.ballot\_tally",

  "options\_ref": "VM-TST-001.options",

  "parameter\_set": {

    "id": "PS:TST020:Baseline:v1",

    "vars": {

      "VM-VAR-001": "approval",

      "VM-VAR-010": "proportional\_favor\_small",

      "VM-VAR-011": "on",

      "VM-VAR-012": 0,

      "VM-VAR-020": 0,

      "VM-VAR-022": 55,

      "VM-VAR-024": "on",

      "VM-VAR-025": "on",

      "VM-VAR-030": "population\_baseline",

      "VM-VAR-031": "country",

      "VM-VAR-040": "none",

      "VM-VAR-050": "status\_quo",

      "VM-VAR-052": 424242

    }

  },

  "acceptance": {

    "across\_os": \["Windows","macOS","Linux"\],

    "require\_identical\_hashes": true

  },

  "expected\_canonical\_hash": null

}

**Notes.** The case doesn’t require randomness, but `VM-VAR-052` is set to fix any incidental RNG usage. Cross-OS equality depends on the determinism rules in Doc 3A/3B (toolchain pinning, canonical serialization).

