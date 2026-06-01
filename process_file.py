
import json
import re

file_path = "/mnt/c/Users/Rishi/Desktop/GV/Adipragya corporation/Devvani/Maharishi_panini/019 Ashtadhyayi Bhashyam-1-Dayananda_text.txt"

# Keywords for extraction
keywords = {
    "sandhi_rules": ["सन्धि", "सन्धिर"],
    "karaka_to_syntax_role": ["कारक", "कर्ता", "कर्म", "करण"],
    "core_dhatus": ["धातु", "धातुपाठ"],
    "pratyaya_patterns": ["प्रत्यय"],
    "vibhakti_cases": ["विभक्ति"],
    "samasa_types": ["समास"],
    "compiler_recommendations": ["व्याकरण", "सूत्र", "लेखन", "प्रयोजन"]
}

results = {key: set() for key in keywords}

def process_file():
    with open(file_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
        
    for i in range(0, len(lines), 500):
        segment = lines[i:i+500]
        segment_text = "\n".join(segment)
        
        for key, patterns in keywords.items():
            for pattern in patterns:
                # Find occurrences. For simplicity, just finding if the word exists
                if pattern in segment_text:
                    # In a real scenario, we'd do more sophisticated parsing
                    results[key].add(pattern)
    
    # Convert sets to lists
    final_results = {key: list(values) for key, values in results.items()}
    print(json.dumps(final_results, ensure_ascii=False, indent=2))

if __name__ == "__main__":
    process_file()
