Search.setIndex({"docnames": ["appendix_chapter1", "appendix_chapter2", "appendix_chapter3", "landing_page", "part1_chapter1", "part1_chapter2", "part1_chapter3", "part1_chapter4", "part1_chapter5", "part2_chapter1", "part2_chapter2", "part2_chapter3", "part2_chapter4", "part2_chapter5", "part3_chapter1", "part3_chapter2", "part3_chapter3", "part3_chapter4"], "filenames": ["appendix_chapter1.md", "appendix_chapter2.md", "appendix_chapter3.md", "landing_page.md", "part1_chapter1.md", "part1_chapter2.md", "part1_chapter3.md", "part1_chapter4.md", "part1_chapter5.ipynb", "part2_chapter1.ipynb", "part2_chapter2.ipynb", "part2_chapter3.ipynb", "part2_chapter4.ipynb", "part2_chapter5.ipynb", "part3_chapter1.ipynb", "part3_chapter2.ipynb", "part3_chapter3.ipynb", "part3_chapter4.ipynb"], "titles": ["Design Principles", "Chiquito vs Halo2", "Chiquito architecture", "Meet Chiquito", "What is Zero Knowledge Proof (Developer POV)?", "What is Chiquito?", "Chiquito Programming Model", "Python Syntax", "Setup", "Chapter 1: Fibonacci and Chiquito Concepts", "Chapter 2: StepType and Circuit", "Chapter 3: Witness", "Chapter 4: Multiple StepTypes", "Chapter 5: Padding and Exposing Signals", "Chapter 1: MiMC7 Concepts", "Chapter 2: First Attempt", "Chapter 3: Witness", "Chapter 4: Fixed Signals, Lookup Tables, and Super Circuit"], "terms": {"abstract": [0, 1, 2, 3, 5, 10], "As": [0, 4, 9, 12, 16], "circuit": [0, 1, 2, 3, 5, 9, 11, 12, 13, 14, 15, 16], "complex": [0, 1, 5], "increas": 0, "inevit": 0, "By": [0, 12, 14], "constraint": [0, 1, 2, 3, 4, 5, 6, 9, 10, 11, 12, 13, 14, 15, 16, 17], "build": [0, 3, 8, 10, 13, 15, 16], "column": [0, 1, 2, 3, 5, 12, 17], "placement": [0, 1, 2, 3], "chiquito": [0, 4, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17], "improv": 0, "readabl": [0, 5], "learnabl": 0, "halo2": [0, 2, 3, 5, 10, 17], "which": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17], "can": [0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17], "onli": [0, 4, 6, 7, 9, 10, 11, 14, 15, 17], "standard": 0, "simplifi": 0, "code": [0, 4, 5, 10, 12, 15, 17], "base": [0, 1, 2, 3, 4, 5, 9, 10, 15], "project": [0, 8, 13, 17], "zkevm": [0, 13, 17], "also": [0, 1, 4, 6, 7, 8, 9, 10, 13, 14, 15, 16, 17], "onboard": 0, "more": [0, 1, 2, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 17], "develop": [0, 2, 5, 8, 12], "compos": [0, 1, 9, 10, 14, 17], "ar": [0, 1, 2, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17], "fulli": [0, 17], "allow": [0, 1, 4, 5, 10, 12, 13], "write": [0, 3, 4, 5, 7, 10, 17], "ani": [0, 5, 6, 7, 10, 11, 16, 17], "part": [0, 4, 6, 7, 9, 14, 15], "entireti": [0, 9], "integr": 0, "other": [0, 4, 6, 10, 14, 15, 16, 17], "modular": [0, 5, 10], "The": [0, 2, 4, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 17], "ast": [0, 2], "ir": [0, 2], "represent": [0, 2], "frontend": [0, 5, 8], "compil": [0, 1, 4], "data": [0, 2, 4, 5, 6, 7, 10, 11], "structur": [0, 2, 3, 5], "backend": [0, 2, 3, 5, 17], "from": [0, 2, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17], "For": [0, 2, 4, 6, 7, 9, 10, 13, 14, 17], "exampl": [0, 2, 4, 6, 7, 8, 9, 10, 12, 13, 14, 15, 17], "we": [0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17], "have": [0, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 16, 17], "python": [0, 2, 3, 4, 5], "sangria": 0, "futur": [0, 2, 5, 17], "extens": [0, 17], "user": [0, 10, 13, 17], "experi": [0, 10], "optim": 0, "annot": [0, 2, 10, 12, 15, 16, 17], "automat": [0, 5], "gener": [0, 1, 4, 5, 6, 7, 9, 11, 12, 13, 15, 16, 17], "debug": [0, 10], "messag": [0, 8, 10, 14, 15, 17], "There": [1, 2, 4, 6, 7, 10, 12, 14, 15], "two": [1, 5, 6, 7, 9, 10, 12, 13, 14, 16, 17], "major": 1, "architectur": 1, "differ": [1, 2, 6, 9, 10, 11, 12, 13, 16, 17], "between": [1, 5, 6, 10, 13, 17], "step": [1, 3, 4, 5, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17], "instanc": [1, 2, 4, 5, 7, 9, 10, 11, 12, 13, 15, 16, 17], "each": [1, 4, 6, 9, 10, 14, 15, 16, 17], "type": [1, 2, 4, 5, 7, 9, 11, 12, 13, 14, 15, 16, 17], "defin": [1, 2, 4, 9, 10, 11, 15, 17], "among": [1, 9, 10, 17], "wit": [1, 4, 5, 7, 9, 12, 13, 15, 17], "fix": [1, 2, 6, 10, 11, 12, 13], "lookup": [1, 2, 5, 6, 10], "tabl": [1, 2, 5, 9, 11, 13, 14, 15, 16], "call": [1, 4, 6, 8, 9, 10, 12, 13, 16, 17], "super": [1, 5], "row": [1, 2, 5, 9, 14, 16, 17], "one": [1, 2, 4, 5, 6, 7, 9, 10, 14, 15, 16, 17], "multipl": [1, 4, 5, 9, 10, 11, 14, 17], "plonkish": [1, 3, 5, 17], "made": [1, 4, 6, 7], "thi": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17], "design": [1, 10, 12, 14], "choic": 1, "sometim": [1, 4, 14], "requir": [1, 4, 8, 10, 14, 16, 17], "alloc": 1, "dsl": [1, 3, 10, 11, 12, 13, 15, 16, 17], "signal": [1, 2, 4, 5, 7, 9, 11, 12, 14, 15, 16], "rather": [1, 13], "than": [1, 2, 4, 13, 15], "order": [1, 4, 5, 6, 16, 17], "awai": 1, "One": [1, 7], "place": [1, 2, 5], "across": [1, 14, 17], "all": [1, 2, 4, 6, 7, 9, 10, 12, 13, 14, 15], "handl": [1, 5], "s": [1, 8, 10, 13, 14, 15, 16, 17], "ha": [1, 2, 3, 4, 5, 6, 10, 16, 17], "follow": [1, 4, 6, 8, 9, 10, 11, 12, 13, 14, 15, 17], "setup": [1, 4, 5, 6, 7, 9, 12, 13, 14, 15, 17], "forward": [1, 2, 5, 6, 7, 10, 11, 12, 13, 14, 15, 16, 17], "share": [1, 2, 5, 6, 7, 9, 10, 17], "intern": [1, 5, 6, 7, 10, 11, 12, 13, 14, 15, 16, 17], "transit": [1, 5, 6, 7, 10, 11, 12, 13, 14, 15, 16, 17], "trace": [1, 4, 5, 7, 11, 12, 13, 15, 16, 17], "implement": [2, 3, 5, 17], "like": [2, 3, 4, 5, 8, 13, 16, 17], "most": [2, 4, 5, 7, 10, 13], "substitut": 2, "lexer": 2, "parser": 2, "declar": [2, 14, 15], "program": 2, "rust": [2, 3, 5], "front": 2, "end": [2, 13, 15, 17], "us": [2, 3, 4, 6, 7, 8, 10, 11, 13, 14, 15, 16, 17], "creat": [2, 4, 6, 7, 8, 10, 12, 13, 14, 15, 16, 17], "an": [2, 4, 6, 7, 8, 9, 10, 12, 13, 15, 16, 17], "syntax": [2, 3, 17], "tree": [2, 13], "repres": [2, 4, 5, 6], "intent": 2, "directli": [2, 15], "put": [2, 7, 12, 15], "inform": [2, 6], "zkp": 2, "alreadi": [2, 5, 10, 17], "contain": [2, 4, 5, 6, 9, 17], "translat": 2, "polynomi": [2, 4], "ident": [2, 12, 17], "In": [2, 4, 5, 6, 7, 8, 9, 10, 11, 13, 14, 15, 17], "want": [2, 5, 13, 16, 17], "do": [2, 7, 9, 16, 17], "so": [2, 4, 5, 6, 11, 17], "effici": [2, 5, 6], "done": [2, 5, 13, 17], "step_typ": [2, 5, 7, 10, 11, 12, 13, 15, 16, 17], "281282176211935008643818839171708619274": 2, "steptyp": [2, 5, 7, 11, 13, 15, 16, 17], "id": [2, 8, 10], "transition_constraint": [2, 10], "transitionconstraint": [2, 10], "b": [2, 4, 5, 7, 9, 10, 11, 12, 13, 17], "next": [2, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 16, 17], "expr": [2, 10], "n": [2, 4, 6, 7, 12, 13, 14, 15, 16, 17], "281282073769920877700029204819473992202": 2, "internalsign": [2, 10], "281282081534280804097934570461757835786": 2, "c": [2, 5, 7, 9, 10, 11, 12, 13, 14, 15, 16, 17], "0x1": [2, 12], "281282153948821342135539412435905153546": 2, "281282157751773142820227898400991480330": 2, "forward_sign": [2, 10], "forwardsign": [2, 10], "281282057290463074733046140937402190346": 2, "phase": [2, 10], "0": [2, 4, 6, 9, 10, 11, 12, 13, 14, 15, 16, 17], "281282068699318476787111035882707749386": 2, "281282071076163352215041445164002970122": 2, "shared_sign": [2, 10], "fixed_sign": [2, 10], "halo2_advic": 2, "halo2_fix": 2, "expos": [2, 5, 7, 8, 10], "last": [2, 7, 13, 15, 16, 17], "fibo": [2, 5, 9, 10, 11, 12, 13], "first": [2, 7, 8, 9, 10, 11, 12, 13, 14, 17], "pad": [2, 5], "fixed_assign": [2, 10], "none": [2, 5, 6, 10, 11, 12], "first_step": [2, 10], "some": [2, 4, 5, 6, 10, 13, 14, 17], "last_step": [2, 10], "num_step": [2, 10], "11": [2, 5, 6, 17], "q_enabl": [2, 10, 12], "true": [2, 6, 10, 12], "number": [2, 4, 5, 6, 7, 9, 10, 13, 14, 15, 16, 17], "three": [2, 4, 9, 10, 11, 13, 16, 17], "advic": [2, 4, 5, 12], "express": [2, 9, 10, 13, 14, 17], "variabl": [2, 6, 9, 14, 17], "rotat": [2, 5, 7, 10, 12, 14], "pair": [2, 4], "1": [2, 4, 5, 6, 7, 10, 11, 12, 13, 15, 16, 17], "2": [2, 4, 5, 7, 9, 11, 12, 13, 14, 16, 17], "offset": [2, 6, 12, 17], "evalu": 2, "zero": [2, 3, 5, 6, 7, 9, 12, 13, 14, 17], "valid": [2, 4, 5, 6, 7], "todo": [2, 10, 12, 13, 17], "diagram": 2, "explain": [2, 5], "better": [2, 3, 5, 14], "take": [2, 10, 13, 15, 16, 17], "produc": 2, "compon": 2, "strategi": [2, 5], "A": [2, 4, 5, 6, 7, 17], "wai": [2, 4, 5, 6, 12], "less": [2, 4], "result": [2, 4, 11, 13, 14, 16, 17], "its": [2, 4, 5, 6, 8, 9, 10, 12, 13, 17], "execut": [2, 4, 6], "112757503094494736886533768524178524682": 2, "signalplac": 2, "mwcm": 2, "ctype": 2, "112757574875209974810026443024764635658": 2, "112757502381441274258154448707306261002": 2, "112757577568967500295014202680235657738": 2, "112757497944664173459351261993868331530": 2, "112757545640018007046483900682209987082": 2, "stepplac": 2, "height": 2, "112757546907668607274713583653889903114": 2, "112757553404377933444389547799470541322": 2, "112757504045232687057706101121682639370": 2, "112757506659762050028429523183609711114": 2, "base_height": 2, "stepselector": 2, "selector_expr": 2, "315264198831617735253233275444124060170": 2, "315264299134471478311887765095509002762": 2, "315264247477709519011537120830062987786": 2, "315264304284302041739069990150842485258": 2, "315264254766700470323856742386059840010": 2, "315264306265006104595678711464417954314": 2, "selector_expr_not": 2, "selector_assign": 2, "0x0000000000000000000000000000000000000000000000000000000000000001": [2, 12], "230476331769351576842702038531557689866": 2, "230477086892968500296103924073925052938": 2, "230476295562081307823898632382065543690": 2, "q_first": 2, "230476296275134770452277952198937807370": 2, "q_last": 2, "230476296671275583023599921641634269706": 2, "230477106700009128862188603934889347594": 2, "230477111453698879718049141022503078410": 2, "230477122070272656629470660032369134090": 2, "poli": 2, "3": [2, 4, 7, 9, 10, 12, 13, 14, 15, 17], "high": [3, 5], "level": [3, 5, 10, 13, 15, 17], "languag": [3, 5, 9, 10], "knowledg": [3, 5, 6, 9, 12, 13, 14, 17], "proof": [3, 5, 6, 9, 10, 11, 13, 14, 16, 17], "current": [3, 5, 7, 10], "being": [3, 5, 17], "work": [3, 4, 5, 7, 8, 17], "group": [3, 5, 17], "pse": [3, 5, 13], "ethereum": [3, 5], "foundat": [3, 5], "It": [3, 4, 6, 10, 14, 16, 17], "zk": [3, 14], "provid": [3, 10, 17], "featur": [3, 17], "low": 3, "zkdsl": 3, "arithemt": 3, "support": 3, "addit": [3, 4, 10, 12, 14, 17], "written": 3, "both": [3, 4, 10, 12, 13, 17], "book": [3, 8], "focus": 3, "nbsp": [3, 5], "mathemat": [4, 9, 14], "comput": [4, 5, 6, 7, 16, 17], "been": [4, 6, 10], "correctli": [4, 6, 10, 16], "while": [4, 5, 6, 7, 13, 15, 17], "reveal": 4, "main": [4, 5, 8, 12, 17], "area": 4, "privaci": [4, 8], "becaus": [4, 9, 10, 11, 12, 13, 14, 15, 16, 17], "fact": [4, 11], "preserv": 4, "secreci": 4, "scalabl": 4, "verifi": [4, 10, 11, 13, 16], "usual": [4, 10, 14], "check": [4, 5, 15, 16, 17], "you": [4, 5, 6, 7, 8, 10, 11, 13, 15, 16, 17], "would": [4, 6, 14, 16, 17], "need": [4, 7, 8, 9, 10, 11, 12, 13, 14, 16, 17], "same": [4, 6, 9, 10, 11, 12, 13, 16, 17], "With": [4, 10, 14, 17], "could": 4, "time": [4, 7, 10, 12, 17], "space": 4, "resourc": 4, "itself": [4, 10], "interoper": 4, "trustless": 4, "organis": 4, "individu": [4, 5], "trust": 4, "without": [4, 5, 17], "particip": 4, "arbitrari": [4, 5, 10, 13, 17], "thei": [4, 5, 6, 9, 10, 13, 14, 15, 16, 17], "definit": 4, "specif": [4, 5, 6, 8, 10, 13, 17], "bubbl": 4, "sort": 4, "algorithm": [4, 6, 7, 14, 15], "mani": [4, 5, 6, 7, 10, 14, 17], "cell": [4, 5], "public": [4, 5], "privat": [4, 13], "input": [4, 5, 6, 7, 10, 13, 14, 15, 16, 17], "output": [4, 5, 6, 13, 14, 15, 16, 17], "intermedi": [4, 5, 6], "valu": [4, 5, 6, 7, 9, 10, 11, 13, 15, 16, 17], "seri": [4, 5, 6, 9, 13, 17], "assert": [4, 5], "over": [4, 10, 12, 13], "help": [4, 16], "calcul": [4, 7, 10, 13, 14], "right": 4, "kei": [4, 10, 14, 15, 16, 17], "given": [4, 5, 10, 17], "prover": [4, 10, 11, 13], "That": [4, 17], "had": [4, 16], "import": [4, 5, 6, 7, 8, 11, 12, 13, 15, 16, 17], "note": [4, 9, 10, 11, 13, 14, 15, 17], "agre": 4, "veri": [4, 5, 13, 14], "simil": 4, "cpu": 4, "machin": [4, 8], "tool": 4, "friendli": [4, 14], "still": [4, 11, 13, 17], "close": 4, "If": [4, 6, 7, 8, 13, 17], "assembl": 4, "closer": 4, "think": [4, 5, 13, 17], "about": [4, 6, 7, 10, 17], "get": [4, 14], "previou": [4, 6], "byte": 4, "understand": [4, 6, 15], "how": [4, 5, 7, 10, 13, 17], "fpf": 4, "see": [4, 5, 6, 8, 10, 15, 17], "unsign": 4, "integ": [4, 17], "where": [4, 10, 13, 14, 17], "maximum": [4, 13], "minu": 4, "p": [4, 14], "instead": [4, 9, 17], "overflow": 4, "expect": [4, 16], "mod": [4, 14], "neg": [4, 10], "obtain": 4, "formula": 4, "big": [4, 10], "when": [4, 7, 13, 15, 17], "easi": [4, 5], "7": [4, 6, 13, 14, 15, 16, 17], "posibl": [4, 6], "4": [4, 9, 10, 11, 13, 16], "5": [4, 9, 10, 11, 12, 16], "6": [4, 11, 12, 13], "go": [4, 8, 15, 17], "around": [4, 15], "Then": [4, 7, 8, 17], "satisfi": [4, 9, 10, 12, 17], "short": 4, "summari": [4, 17], "know": [4, 16], "applic": 5, "dry": 5, "templat": 5, "gadget": 5, "sacrif": 5, "perform": [5, 14], "start": [5, 9, 15, 16, 17], "idea": [5, 6], "everi": [5, 6, 9], "proven": [5, 6], "certain": [5, 6], "divid": 5, "particular": [5, 6, 7], "must": [5, 7, 17], "hold": 5, "set": [5, 6, 7, 8, 9, 10, 11, 13, 14], "function": [5, 7, 10, 11, 13, 14, 15, 16, 17], "anoth": [5, 13, 17], "piec": [5, 12, 13], "element": [5, 6, 10, 15, 16, 17], "enough": 5, "basic": [5, 13], "look": [5, 10, 16, 17], "come": 5, "flavour": 5, "either": 5, "interfac": 5, "chiquitocor": [5, 10], "ad": [5, 9, 10, 11, 12, 13, 16, 17], "new": [5, 11, 12, 13, 15, 17], "out": [5, 10, 15, 16, 17], "builder": [5, 10], "your": [5, 8, 10], "natur": [5, 10, 13, 14], "try": [5, 6, 17], "match": [5, 10, 13, 17], "combin": [5, 7, 10, 13, 17], "easili": [5, 11], "complet": [5, 17], "platform": 5, "readi": [5, 14], "manag": 5, "These": [5, 17], "configur": [5, 7, 10], "find": [5, 16, 17], "selector": [5, 7, 12], "activ": [5, 8], "plan": 5, "nest": 5, "coordin": 5, "boolean": 5, "research": 5, "static": 5, "sound": [5, 11, 12, 15, 16], "issu": 5, "fold": 5, "protostar": 5, "hypernova": 5, "tracer": 5, "But": [5, 7], "yourself": 5, "class": [5, 7, 10, 11, 12, 13, 15, 16, 17], "fibostep": [5, 11, 12, 13], "def": [5, 6, 7, 10, 11, 12, 13, 15, 16, 17], "self": [5, 7, 10, 11, 12, 13, 15, 16, 17], "constr": [5, 7, 10, 11, 12, 13, 15, 16, 17], "eq": [5, 7, 10, 11, 12, 13, 15, 16, 17], "wg": [5, 7, 10, 11, 12, 13, 15, 16, 17], "arg": [5, 10, 11, 12, 13], "tupl": [5, 10, 17], "int": [5, 10], "a_valu": [5, 10, 11, 12, 13], "b_valu": [5, 10, 11, 12, 13], "assign": [5, 7, 9, 10, 11, 12, 13, 15, 16, 17], "f": [5, 10, 11, 12, 13, 15, 16, 17], "queriabl": 5, "fibo_step": [5, 10, 11, 12, 13], "pragma_num_step": [5, 7, 10, 11, 12, 13, 15, 16, 17], "add": [5, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17], "i": [5, 6, 7, 10, 11, 12, 13, 15, 16, 17], "rang": [5, 6, 7, 10, 11, 12, 13, 15, 16, 17], "prev_a": [5, 7, 10, 11, 12, 13], "fibo_wit": [5, 10, 11, 12, 13], "gen_wit": [5, 10, 11, 12, 13, 15, 16, 17], "halo2_mock_prov": [5, 10, 11, 12, 13, 15, 16, 17], "detail": 5, "tutori": [5, 8, 14, 17], "concis": 5, "clear": 5, "best": [6, 9, 13, 16, 17], "analys": 6, "bubblesort": 6, "arr": 6, "len": 6, "j": 6, "actual": [6, 10], "64": 6, "34": [6, 13], "25": [6, 17], "12": 6, "22": 6, "90": [6, 14, 15, 16, 17], "length": 6, "compar": [6, 12, 17], "pre": [6, 14, 16, 17], "state": 6, "befor": [6, 10, 15, 16, 17], "post": 6, "after": [6, 8, 10, 13, 14], "includ": [6, 10, 13], "indent": 6, "rel": [6, 10], "potenti": 6, "should": [6, 7, 8, 10, 13, 15, 16, 17], "plu": [6, 7, 10, 13, 14, 15, 16, 17], "sequenc": 6, "belong": 6, "rule": [6, 7], "whether": 6, "relat": 6, "mean": [6, 7, 9, 10, 13, 14, 15], "cannot": [6, 7, 13, 17], "chang": [6, 8, 14, 16, 17], "possibl": [6, 16], "independ": 6, "paradigm": 6, "them": [6, 9, 10, 16], "refer": [6, 10, 11, 14, 15, 17], "capabl": 6, "involv": [6, 10, 14, 17], "just": [6, 8, 14, 17], "howev": [6, 10, 13, 16, 17], "case": [6, 12, 17], "special": 6, "constant": [6, 12, 14, 15, 16, 17], "scope": 6, "role": 6, "singl": [6, 9, 10, 17], "basi": 7, "arithmet": [7, 17], "oper": [7, 14, 17], "prev": 7, "rot": 7, "sever": 7, "helper": [7, 11], "cb_and": 7, "a1": 7, "And": 7, "cb_or": 7, "Or": 7, "xor": [7, 17], "equal": [7, 10, 13, 17], "select": 7, "when_tru": 7, "when_fals": 7, "trinari": 7, "logic": [7, 17], "impli": [7, 10], "unless": 7, "cb_not": 7, "Not": 7, "isz": 7, "if_next_step": 7, "next_step_must_b": 7, "enforc": [7, 10, 11], "next_step_must_not_b": 7, "version": 7, "arbitrarili": 7, "solut": [7, 17], "operand": 7, "astep": 7, "To": [7, 8, 10, 12, 13, 16, 17], "access": 7, "signal_identifi": 7, "method": 7, "second": [7, 10, 11, 13, 17], "a_val": 7, "b_val": 7, "paramet": [7, 13, 17], "everyth": [7, 12, 15], "togeth": [7, 9, 12, 15], "acircuit": 7, "total": [7, 10, 13, 14, 17], "arg1": 7, "arg2": 7, "a_step": 7, "b_step": 7, "bstep": 7, "c_step": 7, "cstep": 7, "10": [7, 11, 12, 13, 17], "option": [7, 17], "thing": [7, 13], "pragma_first_step": [7, 12, 13, 15, 16, 17], "pragma_last_step": [7, 13, 15, 16, 17], "rest": [7, 13], "needs_pad": [7, 13], "simpli": [8, 10, 17], "run": [8, 15, 17], "line": 8, "command": 8, "pip": 8, "instal": 8, "py": [8, 9, 14], "file": 8, "folder": 8, "depend": [8, 10], "cargo": 8, "toml": 8, "git": 8, "http": 8, "github": 8, "com": 8, "scale": 8, "explor": 8, "rs": 8, "clone": 8, "repo": 8, "navig": 8, "root": [8, 13], "directori": 8, "repositori": 8, "cd": 8, "pyo3": 8, "maturin": 8, "api": [8, 17], "local": 8, "virtual": 8, "environ": 8, "script": 8, "packag": 8, "python3": 8, "m": 8, "venv": 8, "env": 8, "sourc": 8, "bin": 8, "r": 8, "txt": 8, "jupyterlab": 8, "lab": 8, "bind": 8, "abov": [8, 9, 10, 13, 16, 17], "doesn": [8, 13, 14, 17], "t": [8, 10, 11, 12, 13, 14, 16, 17], "guid": 8, "here": [8, 10, 15, 17], "getting_start": 8, "test": [8, 10, 11, 16], "success": [8, 11, 15, 16, 17], "fibonacci": [8, 10, 11, 12, 13, 15, 17], "mimc7": [8, 15, 16, 17], "correct": 8, "ok": [8, 10, 11, 12, 13, 15, 16, 17], "print": [8, 10, 11, 13, 16, 17], "termin": 8, "modifi": [8, 11, 13, 17], "contribut": 8, "ipynb": 8, "vscode": 8, "altern": [8, 17], "browser": 8, "accord": [8, 11, 13], "make": [8, 13, 17], "sure": [8, 17], "ve": [8, 17], "up": [8, 10, 14, 17], "direct": 8, "kernel": 8, "chiquito_kernel": 8, "ipykernel": 8, "name": [8, 10, 12, 17], "subfold": 8, "launch": 8, "notebook": 8, "tab": 8, "top": [8, 13], "menu": 8, "bar": 8, "click": 8, "readm": 8, "NOT": 8, "learn": [9, 10, 12, 13, 17], "walk": [9, 14], "through": [9, 10, 13, 14, 15, 17], "infinit": 9, "sum": [9, 10, 12], "preced": 9, "few": [9, 13], "round": [9, 13, 14, 15, 16, 17], "8": [9, 10, 11, 12, 13, 17], "therefor": [9, 10, 11, 14, 17], "equat": [9, 14], "typic": 9, "matrix": 9, "form": [9, 17], "construct": [9, 10, 12, 13, 14, 15, 16], "besid": [9, 12, 13], "relationship": [9, 13, 14, 17], "separ": [9, 12, 17], "smaller": 9, "chunk": 9, "collect": 9, "indic": 9, "index": [9, 10, 11, 12, 13, 16, 17], "although": [9, 13, 17], "essenti": [9, 10, 16, 17], "instanti": [9, 10, 12, 13, 17], "becom": 9, "simplic": 9, "below": [9, 11, 17], "recap": 9, "subset": 9, "concept": [10, 15, 17], "pychiquito": 10, "let": [10, 13, 14, 15, 16, 17], "domain": 10, "cb": [10, 11, 12, 13, 15, 16, 17], "field": [10, 14, 15, 17], "util": [10, 11, 12, 13, 15, 16, 17], "rememb": [10, 15], "parent": 10, "customarili": 10, "inherit": [10, 17], "now": [10, 11, 13, 15, 16, 17], "bit": [10, 14, 17], "queri": 10, "non": [10, 13, 14], "were": [10, 11, 16, 17], "respect": [10, 16], "ever": 10, "posit": [10, 11, 12], "well": [10, 13, 16, 17], "within": [10, 17], "didn": [10, 11, 16], "later": [10, 13, 17], "stand": 10, "e": [10, 12, 17], "etc": 10, "snippet": [10, 17], "wherea": [10, 12], "argument": [10, 17], "pass": [10, 11, 12, 13, 15, 16, 17], "wrap": [10, 15], "convert": [10, 17], "finish": [10, 17], "object": [10, 17], "previous": [10, 17], "append": [10, 12, 13, 17], "constructor": 10, "final": [10, 12, 13, 14, 15, 17], "constrain": [10, 11, 12, 13, 14, 15, 17], "hardcod": 10, "associ": 10, "earlier": 10, "g": [10, 12], "loop": [10, 17], "voila": [10, 15, 17], "our": [10, 11, 13, 14, 15, 16, 17], "went": 10, "random": 10, "uuid": 10, "uniqu": 10, "identifi": 10, "don": [10, 16, 17], "worri": 10, "astcircuit": 10, "188490972693979613829115140347282459146": 10, "aststeptyp": 10, "188490979349145265028215996005797726730": 10, "188490953203851635320302576267724196362": 10, "188490965959585800117303970441642773002": 10, "initi": [10, 11, 13, 15, 17], "extern": 10, "wa": [10, 11], "practic": [10, 13, 16, 17], "why": [10, 11, 17], "again": [10, 12, 13, 15, 16, 17], "tracewit": [10, 11, 13], "step_inst": [10, 11, 13, 16], "stepinst": [10, 11, 13, 16], "step_type_uuid": [10, 11, 13, 16], "mock": 10, "err": [10, 12], "error": [10, 11, 12], "congratul": 10, "behind": [10, 17], "smooth": 10, "tenet": 10, "never": [10, 11], "easier": 10, "intend": [11, 17], "accept": 11, "context": [11, 13], "faulti": 11, "successfulli": [11, 16], "against": [11, 15], "fals": [11, 12], "incur": 11, "swap": [11, 16], "evil_witness_test": [11, 12, 16], "evil_wit": [11, 12, 16], "step_instance_indic": [11, 12, 16], "assignment_indic": [11, 12, 16], "rh": [11, 12, 16, 17], "third": [11, 13], "fourth": 11, "likewis": 11, "displai": 11, "further": [11, 17], "confirm": [11, 15, 16, 17], "189540066703484544027986316683742153226": 11, "surprisingli": 11, "verif": 11, "constitut": 11, "isn": [11, 12, 17], "specifi": [11, 13], "answer": [11, 17], "simpl": [11, 14, 17], "might": [11, 13, 17], "wonder": [11, 13, 17], "realli": [11, 13], "whose": [11, 17], "tamper": [11, 13], "shown": 11, "condit": 11, "avoid": 12, "fibofirststep": [12, 13], "identi": 12, "except": [12, 17], "show": 12, "increment": [12, 17], "otherwis": 12, "fibo_first_step": [12, 13], "fail": [12, 16], "wrote": 12, "constraintcasedebug": 12, "gate": 12, "product": 12, "query_index": 12, "column_index": 12, "negat": 12, "locat": 12, "inregion": 12, "region": 12, "cell_valu": 12, "debugvirtualcel": 12, "debugcolumn": 12, "column_typ": 12, "srcm": 12, "0x2": 12, "elimin": 12, "demonstr": [12, 13], "iter": 12, "engin": 12, "proactiv": 12, "edg": 12, "prevent": 12, "therebi": [12, 13], "great": [12, 16, 17], "versatil": 12, "process": 12, "analog": [12, 13], "consid": [12, 13], "hardwar": [12, 13], "softwar": [12, 13], "compat": [12, 13], "prior": [13, 15, 17], "d": 13, "flexibl": 13, "immedi": 13, "shouldn": 13, "limit": 13, "guarante": 13, "secur": 13, "AND": [13, 17], "problem": 13, "frequent": 13, "encount": 13, "size": [13, 14], "achiev": [13, 17], "techniqu": 13, "won": 13, "exceed": 13, "plausibl": 13, "toward": 13, "9": 13, "13": 13, "21": 13, "default": 13, "visibl": 13, "writer": 13, "common": [13, 14, 17], "system": [13, 17], "hash": [13, 14, 15, 16, 17], "merkl": 13, "pure": 13, "purpos": [13, 17], "sai": [13, 17], "copi": 13, "popul": 13, "introduc": [13, 15, 17], "n_valu": 13, "leav": 13, "ll": 13, "correspond": [13, 16, 17], "exist": 13, "account": 13, "did": [13, 17], "1st": [13, 14, 15, 16], "2nd": [13, 14, 15, 16], "th": 13, "fill": 13, "fewer": 13, "chiquito_ast": 13, "192118270666995939598363260919147530762": 13, "192118326047481537072691112187154532874": 13, "192118355758042479918876577734486264330": 13, "return": 13, "another_fibo_wit": 13, "custom": [13, 17], "minim": 14, "sha256": 14, "costli": 14, "term": 14, "x": [14, 15, 16, 17], "k": [14, 15, 16, 17], "c_i": 14, "determin": [14, 16], "list": [14, 17], "distinct": [14, 15, 17], "f_i": 14, "f_": 14, "91": [14, 15, 16, 17], "intuit": 14, "furthermor": [14, 16, 17], "xkc": [14, 15, 16, 17], "y": [14, 15, 16, 17], "0th": [14, 15, 16], "90th": [14, 15, 16], "feel": 14, "plug": 14, "huge": [14, 15], "toi": 14, "assum": 14, "c_0": 14, "c_1": 14, "c_2": 14, "c_3": 14, "3rd": [14, 15, 16], "2183": [14, 15, 16], "2186": [14, 15, 16], "238534446168822298080896": 14, "238534446168822298080897": 14, "13936292545083981204475079684474676334933008887368755327375281694396644515858": 14, "due": 14, "exponenti": [14, 16], "blown": 14, "quickli": 14, "prime": 14, "modulu": 14, "2385": [14, 15, 16], "0896": [14, 15, 16], "0897": [14, 15, 16], "1393": [14, 15, 16], "5858": [14, 15, 16], "4th": [14, 15, 16], "89th": [14, 15, 16], "y_90": 14, "x_": [14, 15, 16], "y_": [14, 15, 16], "mimc7step": [14, 15, 16, 17], "mimc7laststep": [14, 15, 16, 17], "addition": [14, 17], "attempt": 14, "ones": 15, "highli": 15, "recommend": 15, "x_valu": [15, 16, 17], "k_valu": [15, 16, 17], "c_valu": [15, 16, 17], "xkc_valu": [15, 16, 17], "mimclaststep": 15, "_": [15, 16, 17], "mimc7_step": [15, 16, 17], "mimc7_last_step": [15, 16, 17], "denot": 15, "extra": 15, "x_in_valu": [15, 16, 17], "arrai": [15, 16, 17], "round_const": [15, 16, 17], "peek": 15, "curiou": 15, "warn": 15, "mimc7circuit": [15, 16], "__future__": [15, 16, 17], "20888961410941983456478427210666206549300505294776164667214940546594746570981": [15, 16, 17], "15265126113435022738560151911929040668591755459209400716467504685752745317193": [15, 16, 17], "8334177627492981984476504167502758309043212251641796197711684499645635709656": [15, 16, 17], "1374324219480165500871639364801692115397519265181803854177629327624133579404": [15, 16, 17], "11442588683664344394633565859260176446561886575962616332903193988751292992472": [15, 16, 17], "2558901189096558760448896669327086721003508630712968559048179091037845349145": [15, 16, 17], "11189978595292752354820141775598510151189959177917284797737745690127318076389": [15, 16, 17], "3262966573163560839685415914157855077211340576201936620532175028036746741754": [15, 16, 17], "17029914891543225301403832095880481731551830725367286980611178737703889171730": [15, 16, 17], "4614037031668406927330683909387957156531244689520944789503628527855167665518": [15, 16, 17], "19647356996769918391113967168615123299113119185942498194367262335168397100658": [15, 16, 17], "5040699236106090655289931820723926657076483236860546282406111821875672148900": [15, 16, 17], "2632385916954580941368956176626336146806721642583847728103570779270161510514": [15, 16, 17], "17691411851977575435597871505860208507285462834710151833948561098560743654671": [15, 16, 17], "11482807709115676646560379017491661435505951727793345550942389701970904563183": [15, 16, 17], "8360838254132998143349158726141014535383109403565779450210746881879715734773": [15, 16, 17], "12663821244032248511491386323242575231591777785787269938928497649288048289525": [15, 16, 17], "3067001377342968891237590775929219083706800062321980129409398033259904188058": [15, 16, 17], "8536471869378957766675292398190944925664113548202769136103887479787957959589": [15, 16, 17], "19825444354178182240559170937204690272111734703605805530888940813160705385792": [15, 16, 17], "16703465144013840124940690347975638755097486902749048533167980887413919317592": [15, 16, 17], "13061236261277650370863439564453267964462486225679643020432589226741411380501": [15, 16, 17], "10864774797625152707517901967943775867717907803542223029967000416969007792571": [15, 16, 17], "10035653564014594269791753415727486340557376923045841607746250017541686319774": [15, 16, 17], "3446968588058668564420958894889124905706353937375068998436129414772610003289": [15, 16, 17], "4653317306466493184743870159523234588955994456998076243468148492375236846006": [15, 16, 17], "8486711143589723036499933521576871883500223198263343024003617825616410932026": [15, 16, 17], "250710584458582618659378487568129931785810765264752039738223488321597070280": [15, 16, 17], "2104159799604932521291371026105311735948154964200596636974609406977292675173": [15, 16, 17], "16313562605837709339799839901240652934758303521543693857533755376563489378839": [15, 16, 17], "6032365105133504724925793806318578936233045029919447519826248813478479197288": [15, 16, 17], "14025118133847866722315446277964222215118620050302054655768867040006542798474": [15, 16, 17], "7400123822125662712777833064081316757896757785777291653271747396958201309118": [15, 16, 17], "1744432620323851751204287974553233986555641872755053103823939564833813704825": [15, 16, 17], "8316378125659383262515151597439205374263247719876250938893842106722210729522": [15, 16, 17], "6739722627047123650704294650168547689199576889424317598327664349670094847386": [15, 16, 17], "21211457866117465531949733809706514799713333930924902519246949506964470524162": [15, 16, 17], "13718112532745211817410303291774369209520657938741992779396229864894885156527": [15, 16, 17], "5264534817993325015357427094323255342713527811596856940387954546330728068658": [15, 16, 17], "18884137497114307927425084003812022333609937761793387700010402412840002189451": [15, 16, 17], "5148596049900083984813839872929010525572543381981952060869301611018636120248": [15, 16, 17], "19799686398774806587970184652860783461860993790013219899147141137827718662674": [15, 16, 17], "19240878651604412704364448729659032944342952609050243268894572835672205984837": [15, 16, 17], "10546185249390392695582524554167530669949955276893453512788278945742408153192": [15, 16, 17], "5507959600969845538113649209272736011390582494851145043668969080335346810411": [15, 16, 17], "18177751737739153338153217698774510185696788019377850245260475034576050820091": [15, 16, 17], "19603444733183990109492724100282114612026332366576932662794133334264283907557": [15, 16, 17], "10548274686824425401349248282213580046351514091431715597441736281987273193140": [15, 16, 17], "1823201861560942974198127384034483127920205835821334101215923769688644479957": [15, 16, 17], "11867589662193422187545516240823411225342068709600734253659804646934346124945": [15, 16, 17], "18718569356736340558616379408444812528964066420519677106145092918482774343613": [15, 16, 17], "10530777752259630125564678480897857853807637120039176813174150229243735996839": [15, 16, 17], "20486583726592018813337145844457018474256372770211860618687961310422228379031": [15, 16, 17], "12690713110714036569415168795200156516217175005650145422920562694422306200486": [15, 16, 17], "17386427286863519095301372413760745749282643730629659997153085139065756667205": [15, 16, 17], "2216432659854733047132347621569505613620980842043977268828076165669557467682": [15, 16, 17], "6309765381643925252238633914530877025934201680691496500372265330505506717193": [15, 16, 17], "20806323192073945401862788605803131761175139076694468214027227878952047793390": [15, 16, 17], "4037040458505567977365391535756875199663510397600316887746139396052445718861": [15, 16, 17], "19948974083684238245321361840704327952464170097132407924861169241740046562673": [15, 16, 17], "845322671528508199439318170916419179535949348988022948153107378280175750024": [15, 16, 17], "16222384601744433420585982239113457177459602187868460608565289920306145389382": [15, 16, 17], "10232118865851112229330353999139005145127746617219324244541194256766741433339": [15, 16, 17], "6699067738555349409504843460654299019000594109597429103342076743347235369120": [15, 16, 17], "6220784880752427143725783746407285094967584864656399181815603544365010379208": [15, 16, 17], "6129250029437675212264306655559561251995722990149771051304736001195288083309": [15, 16, 17], "10773245783118750721454994239248013870822765715268323522295722350908043393604": [15, 16, 17], "4490242021765793917495398271905043433053432245571325177153467194570741607167": [15, 16, 17], "19596995117319480189066041930051006586888908165330319666010398892494684778526": [15, 16, 17], "837850695495734270707668553360118467905109360511302468085569220634750561083": [15, 16, 17], "11803922811376367215191737026157445294481406304781326649717082177394185903907": [15, 16, 17], "10201298324909697255105265958780781450978049256931478989759448189112393506592": [15, 16, 17], "13564695482314888817576351063608519127702411536552857463682060761575100923924": [15, 16, 17], "9262808208636973454201420823766139682381973240743541030659775288508921362724": [15, 16, 17], "173271062536305557219323722062711383294158572562695717740068656098441040230": [15, 16, 17], "18120430890549410286417591505529104700901943324772175772035648111937818237369": [15, 16, 17], "20484495168135072493552514219686101965206843697794133766912991150184337935627": [15, 16, 17], "19155651295705203459475805213866664350848604323501251939850063308319753686505": [15, 16, 17], "11971299749478202793661982361798418342615500543489781306376058267926437157297": [15, 16, 17], "18285310723116790056148596536349375622245669010373674803854111592441823052978": [15, 16, 17], "7069216248902547653615508023941692395371990416048967468982099270925308100727": [15, 16, 17], "6465151453746412132599596984628739550147379072443683076388208843341824127379": [15, 16, 17], "16143532858389170960690347742477978826830511669766530042104134302796355145785": [15, 16, 17], "19362583304414853660976404410208489566967618125972377176980367224623492419647": [15, 16, 17], "1702213613534733786921602839210290505213503664731919006932367875629005980493": [15, 16, 17], "10781825404476535814285389902565833897646945212027592373510689209734812292327": [15, 16, 17], "4212716923652881254737947578600828255798948993302968210248673545442808456151": [15, 16, 17], "7594017890037021425366623750593200398174488805473151513558919864633711506220": [15, 16, 17], "18979889247746272055963929241596362599320706910852082477600815822482192194401": [15, 16, 17], "13602139229813231349386885113156901793661719180900395818909719758150455500533": [15, 16, 17], "mimc7_circuit": [15, 16, 17], "mimc7_circuit_wit": [15, 16], "appli": [15, 17], "proce": 15, "evil": [16, 17], "alwai": 16, "review": 16, "small": 16, "signific": 16, "larg": 16, "string": 16, "caus": 16, "cascad": 16, "origin": [16, 17], "195622914491657820062758153102071302666": 16, "19772642601925508232386889125207430697825779573800034433688041678604067403935": 16, "11486538959899464359527368493107057402939134354284395908899557250178714408853": 16, "10594780656576967754230020536574539122676596303354946869887184401991294982662": 16, "195622968921405467364112231921625991690": 16, "10594780656576967754230020536574539122676596303354946869887184401991294982664": 16, "c_row90": 16, "xkc_row90": 16, "y_row90": 16, "5th": 16, "x_row91": 16, "out_row91": 16, "3th": 16, "19772642601925508232386889125207430697825779573800034433688041678604067403938": 16, "2038891600805023480257114900259151954044463841564468389732305079234997849132": 16, "2038891600805023480257114900259151954044463841564468389732305079234997849134": 16, "exactli": [16, 17], "significantli": 16, "hope": 16, "what": [16, 17], "happen": 16, "turn": [16, 17], "quit": [16, 17], "difficult": 16, "accommod": 16, "keep": 16, "read": 16, "whole": 17, "enabl": 17, "suppos": 17, "certainli": 17, "manual": 17, "lengthi": 17, "known": 17, "good": 17, "bitwis": 17, "00": 17, "01": 17, "decompos": 17, "32": 17, "reduc": 17, "wide": 17, "routin": 17, "sha": 17, "256": 17, "back": 17, "recal": 17, "92": 17, "occupi": 17, "even": 17, "long": 17, "parallel": 17, "wouldn": 17, "wast": 17, "giant": 17, "scene": 17, "mouth": 17, "singleton": 17, "content": 17, "regular": 17, "fixed_gen": 17, "similar": 17, "invok": 17, "cicuit": 17, "lookup_row": 17, "lookup_c": 17, "under": 17, "new_tabl": 17, "twice": 17, "lookup_t": 17, "doe": 17, "via": 17, "lh": 17, "contrarili": 17, "round_kei": 17, "enumer": 17, "necessari": 17, "duplic": 17, "reason": 17, "appear": 17, "2n": 17, "settl": 17, "mimc7firststep": 17, "mention": 17, "abbrevi": 17, "row_valu": 17, "add_lookup": 17, "accomplish": 17, "insid": 17, "grab": 17, "yet": 17, "trick": 17, "store": 17, "resid": 17, "eventu": 17, "constants_t": 17, "chain": 17, "full": 17, "conveni": 17, "spell": 17, "though": 17, "mimc7_first_step": 17, "feed": 17, "document": 17, "far": 17, "wire": 17, "core": 17, "often": 17, "supercircuit": 17, "similarli": 17, "map": 17, "talk": 17, "TO": 17, "sub_circuit": 17, "unlimit": 17, "super_circuit": 17, "mimc7supercircuit": 17, "mimc7_const": 17, "point": 17, "inde": 17, "down": 17, "pipelin": 17, "onc": 17, "scroll": 17, "progress": 17, "give": 17, "mimc7_super_wit": 17, "hypothet": 17, "scenario": 17, "ensur": 17, "reflect": 17, "lot": 17, "alon": 17, "entir": 17, "becam": 17, "newli": 17, "mint": 17, "expert": 17}, "objects": {}, "objtypes": {}, "objnames": {}, "titleterms": {"design": 0, "principl": 0, "chiquito": [1, 2, 3, 5, 6, 9], "vs": 1, "halo2": 1, "architectur": 2, "plonkish": 2, "arithmet": [2, 4], "compil": 2, "cell": 2, "manag": 2, "step": [2, 6, 7], "selector": 2, "meet": 3, "what": [4, 5], "zero": 4, "knowledg": 4, "proof": 4, "develop": 4, "pov": 4, "ar": [4, 5], "applic": 4, "zkp": [4, 5, 6], "prove": [4, 5], "system": [4, 5], "element": 4, "circuit": [4, 6, 7, 10, 17], "low": 4, "level": 4, "dsl": [4, 5], "high": 4, "structur": 4, "languag": 4, "finit": 4, "prime": 4, "field": 4, "why": 5, "differ": 5, "from": 5, "other": 5, "program": [5, 6], "model": [5, 6], "us": 5, "featur": 5, "fibonacci": [5, 9], "circtuit": 5, "pychiquito": 5, "trace": [6, 10], "checker": 6, "wit": [6, 10, 11, 16], "type": [6, 10], "instanc": 6, "signal": [6, 10, 13, 17], "rotat": 6, "put": [6, 10, 13, 17], "everyth": [6, 10, 13, 17], "togeth": [6, 10, 13, 17], "python": [7, 8], "syntax": 7, "constraint": 7, "builder": 7, "defin": 7, "setup": [8, 10, 11, 16], "user": 8, "rust": 8, "contributor": 8, "jupyt": 8, "chapter": [9, 10, 11, 12, 13, 14, 15, 16, 17], "1": [9, 14], "concept": [9, 13, 14], "2": [10, 15], "steptyp": [10, 12], "import": 10, "fibostep": 10, "gener": 10, "3": [11, 16], "4": [12, 17], "multipl": 12, "5": 13, "pad": 13, "expos": 13, "code": 13, "up": 13, "mimc7": 14, "first": 15, "attempt": 15, "fix": 17, "lookup": 17, "tabl": 17, "super": 17, "them": 17, "all": 17, "mimc7const": 17, "sub": 17, "modif": 17, "mimc7circuit": 17, "construct": 17}, "envversion": {"sphinx.domains.c": 2, "sphinx.domains.changeset": 1, "sphinx.domains.citation": 1, "sphinx.domains.cpp": 6, "sphinx.domains.index": 1, "sphinx.domains.javascript": 2, "sphinx.domains.math": 2, "sphinx.domains.python": 3, "sphinx.domains.rst": 2, "sphinx.domains.std": 2, "sphinx.ext.intersphinx": 1, "sphinx": 56}})