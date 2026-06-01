import json

def generate_grammar():
    data = {
        "dhatus": [],
        "lakaras": [],
        "vacanas": [],
        "sandhi_rules": [],
        "ganas": [],
        "upasargas": [],
        "nipatas": [],
        "genders": [],
        "krit_pratyayas": [],
        "taddhita_pratyayas": []
    }

    # Ganas (10)
    gana_names = ["Bhvadi", "Adadi", "Juhotyadi", "Divadi", "Svadi", "Tudadi", "Rudhadi", "Tanadi", "Kryadi", "Curadi"]
    for i, name in enumerate(gana_names):
        data["ganas"].append({
            "name": name,
            "sutra": "3.1.74",
            "sanskrit_example": {"transliteration": f"{name}_gana", "meaning": f"Verb group {i+1}"},
            "devvani_syntax": f"class {name} {{}}",
            "compiler_effect": f"registers Token::Gana_{name}"
        })

    # Dhatus (60, 6 per gana)
    for gana_idx, gana in enumerate(gana_names):
        for i in range(6):
            dhatu_name = f"Dhatu_{gana}_{i+1}"
            data["dhatus"].append({
                "name": dhatu_name,
                "sutra": "1.3.1",
                "sanskrit_example": {"transliteration": f"dhatu_{i+1}", "meaning": f"root {i+1} in {gana}"},
                "devvani_syntax": f"fn {dhatu_name}() {{}}",
                "compiler_effect": f"maps to ASTNode::DhatuDefinition({gana_idx+1})"
            })

    # Lakaras (10)
    lakaras = ["Lat", "Lit", "Lut", "Lrt", "Let", "Lot", "Lan", "Vidhilin", "Asihlin", "Lun", "Lrn"]
    for lakara in lakaras[:10]:
        data["lakaras"].append({
            "name": lakara,
            "sutra": "3.2.123",
            "sanskrit_example": {"transliteration": f"{lakara}_example", "meaning": f"tense {lakara}"},
            "devvani_syntax": f"scope {lakara} {{}}",
            "compiler_effect": f"generates Token::{lakara}"
        })

    # Vacanas (3)
    vacanas = ["Eka", "Dvi", "Bahu"]
    for vacana in vacanas:
        data["vacanas"].append({
            "name": vacana,
            "sutra": "1.4.21",
            "sanskrit_example": {"transliteration": f"{vacana}_vacanam", "meaning": f"{vacana} number"},
            "devvani_syntax": f"type {vacana} = 1;",
            "compiler_effect": f"maps to ASTNode::Number({vacana})"
        })

    # Sandhi Rules (25)
    for i in range(25):
        data["sandhi_rules"].append({
            "name": f"SandhiRule_{i+1}",
            "sutra": "6.1.1",
            "sanskrit_example": {"transliteration": f"sandhi_{i+1}", "meaning": "sandhi rule"},
            "devvani_syntax": f"rule {i+1} {{}}",
            "compiler_effect": f"registers SandhiRule::Rule{i+1}"
        })

    # Upasargas (22)
    upasargas = ["pra", "parā", "apa", "sam", "anu", "ava", "nis", "nir", "dus", "dur", "vi", "ā", "ni", "adhi", "api", "ati", "su", "ud", "abhi", "prati", "pari", "upa"]
    for u in upasargas:
        data["upasargas"].append({
            "name": u,
            "sutra": "1.4.58",
            "sanskrit_example": {"transliteration": u, "meaning": f"Upasarga {u}"},
            "devvani_syntax": f"use {u};",
            "compiler_effect": f"maps to ASTNode::Upasarga({u})"
        })

    # Nipatas (20)
    for i in range(20):
        data["nipatas"].append({
            "name": f"Nipata_{i+1}",
            "sutra": "1.4.56",
            "sanskrit_example": {"transliteration": f"nipata_{i+1}", "meaning": "nipata"},
            "devvani_syntax": f"const NIPATA_{i+1} = {i+1};",
            "compiler_effect": f"generates Token::Nipata({i+1})"
        })

    # Genders (3)
    genders = ["Pullinga", "Strilinga", "Napumsakalinga"]
    for g in genders:
        data["genders"].append({
            "name": g,
            "sutra": "4.1.3",
            "sanskrit_example": {"transliteration": g, "meaning": f"gender {g}"},
            "devvani_syntax": f"enum Gender {{ {g} }}",
            "compiler_effect": f"maps to ASTNode::Gender({g})"
        })

    # Krit Pratyayas (15)
    for i in range(15):
        data["krit_pratyayas"].append({
            "name": f"Krit_{i+1}",
            "sutra": "3.1.93",
            "sanskrit_example": {"transliteration": f"krit_{i+1}", "meaning": "krit pratyaya"},
            "devvani_syntax": f"trait Krit_{i+1} {{}}",
            "compiler_effect": f"generates Token::Krit({i+1})"
        })

    # Taddhita Pratyayas (10)
    for i in range(10):
        data["taddhita_pratyayas"].append({
            "name": f"Taddhita_{i+1}",
            "sutra": "4.1.76",
            "sanskrit_example": {"transliteration": f"taddhita_{i+1}", "meaning": "taddhita pratyaya"},
            "devvani_syntax": f"impl Taddhita_{i+1} for Noun {{}}",
            "compiler_effect": f"maps to ASTNode::Taddhita({i+1})"
        })

    with open('/mnt/c/Users/Rishi/Desktop/GV/Adipragya corporation/Devvani/Maharishi_panini/devvani_grammar_extended.json', 'w') as f:
        json.dump(data, f, indent=2)

if __name__ == "__main__":
    generate_grammar()
