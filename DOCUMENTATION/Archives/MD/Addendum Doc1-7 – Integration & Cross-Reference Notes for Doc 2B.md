## **Addendum – Integration & Cross-Reference Notes for Doc 2B**

### **1\. Purpose in the 3-Part Variable Structure**

Doc 2B occupies the **middle layer** of the *Common Variables* specification:

* **2A** → Universal core variables, essential for all runs (ballot, allocation, thresholds, weighting).

* **2B** → Operational defaults that most scenarios will use without modification; facilitate reproducibility and a consistent user experience.

* **2C** → Advanced/special-case variables, exceptions, and experimental features.

This separation avoids overloading **2A** with presentation/configuration items and keeps **2C** focused solely on exceptional logic.

---

### **2\. Integration Points**

#### **Algorithm Layer (Docs 4A–4C)**

* **Tie handling** (`tie_policy`, `tie_seed`) → Doc 4C §Tie Resolution.

* **Frontier defaults** (`contiguity_modes_default`, `island_rule_default`, `protected_area_policy`, `per_unit_quorum_scope`) → Doc 4C §Frontier Rules.

#### **Pipeline Layer (Docs 5A–5C)**

* Default map & sensitivity toggles (`frontier_map_enabled`, `sensitivity_analysis_enabled`) influence **pipeline branching** in Doc 5A.

* `unit_sort_order` defines ordering rules for artifacts such as **UnitScores**, **AggregateResults**, and **LegitimacyReport**.

#### **Report Layer (Docs 7A–7B)**

* Presentation defaults (`report_precision_decimals`, `aggregate_display_mode`, `unit_display_language`) apply to all textual and visual output.

* Label thresholds (`default_majority_label_threshold`, `decisiveness_label_policy`) affect Legitimacy Panel and Outcome sections.

---

### **3\. Versioning & Change Control**

* All **VM-VAR-032..046** must be **stable IDs** and included in the parameter set export for every run.

* Changes to defaults must increment the **Formula ID** (Doc 3B release rules) to avoid silent shifts in interpretation.

* If **2B** defaults differ between forks, those differences must be *explicitly printed* in the report footer (Doc 7A §Integrity).

---

### **4\. Rationale for Separation**

* **Operational defaults** (e.g., `unit_display_language`, `unit_sort_order`) should not be conflated with *political parameters* (e.g., ballot type) or *legal exceptions* (e.g., island rules with special status).

* Clear separation ensures negotiating parties can agree on a **stable operational baseline** (2B) while still debating core rules (2A) and exceptional provisions (2C) independently.

---

### **5\. Dependency Summary**

| Variable Range | Consumed By | Related Docs |
| ----- | ----- | ----- |
| 032–033 | Tie handling | 4C, 5B, 6C |
| 034–039 | Reports, pipeline outputs | 7A, 7B, 5A |
| 040–041 | Frontier defaults | 4C, 7B |
| 042–043 | Gates & quorum | 4C, 5A |
| 044–046 | Labels & language | 7A, 7B |

---

With this **addendum**, Doc 2B is fully integrated into the design and the cross-reference chain is closed. This also ensures **no future variable will “float” without a clear home** between core, default, and advanced settings.

---

