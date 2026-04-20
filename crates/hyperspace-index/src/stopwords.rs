//! Built-in stopword lists for 17 languages.
//!
//! Provides default stopword sets derived from NLTK/Snowball for common languages.
//! These can be passed to the [`Tokenizer`](crate::tokenizer::Tokenizer) builder.
//!
//! Each language has:
//! - A `pub const LANG: &[&str]` array of stopwords
//! - A `pub fn lang() -> Vec<String>` convenience accessor returning owned strings

/// English stopwords (NLTK-derived set).
pub const ENGLISH: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "if", "in", "into", "is", "it",
    "no", "not", "of", "on", "or", "such", "that", "the", "their", "then", "there", "these",
    "they", "this", "to", "was", "will", "with",
];

/// German stopwords (Snowball-derived set).
pub const GERMAN: &[&str] = &[
    "aber",
    "alle",
    "allem",
    "allen",
    "aller",
    "allerdings",
    "alles",
    "also",
    "am",
    "an",
    "ander",
    "andere",
    "anderem",
    "anderen",
    "anderer",
    "anderes",
    "anderm",
    "andern",
    "anders",
    "auch",
    "auf",
    "aus",
    "bei",
    "beim",
    "bereits",
    "bin",
    "bis",
    "bist",
    "da",
    "dabei",
    "dadurch",
    "dafuer",
    "dagegen",
    "daher",
    "dahin",
    "damals",
    "damit",
    "danach",
    "daneben",
    "dann",
    "daran",
    "darauf",
    "daraus",
    "darf",
    "darfst",
    "darin",
    "darueber",
    "darum",
    "darunter",
    "das",
    "dasselbe",
    "dass",
    "davon",
    "davor",
    "dazu",
    "dein",
    "deine",
    "deinem",
    "deinen",
    "deiner",
    "dem",
    "den",
    "denn",
    "dennoch",
    "der",
    "deren",
    "des",
    "deshalb",
    "dessen",
    "die",
    "dies",
    "diese",
    "dieselbe",
    "dieselben",
    "diesem",
    "diesen",
    "dieser",
    "dieses",
    "doch",
    "dort",
    "drei",
    "du",
    "durch",
    "duerfen",
    "ein",
    "eine",
    "einem",
    "einen",
    "einer",
    "einige",
    "einigem",
    "einigen",
    "einiger",
    "einiges",
    "einmal",
    "er",
    "es",
    "etwas",
    "euch",
    "euer",
    "eure",
    "eurem",
    "euren",
    "eurer",
];

/// French stopwords (Snowball-derived set).
pub const FRENCH: &[&str] = &[
    "au", "aux", "avec", "ce", "ces", "dans", "de", "des", "du", "elle", "en", "et", "eux", "il",
    "je", "la", "le", "les", "leur", "lui", "ma", "mais", "me", "mes", "moi", "mon", "ne", "nos",
    "notre", "nous", "on", "ou", "par", "pas", "pour", "qu", "que", "qui", "sa", "se", "ses", "si",
    "son", "sur", "ta", "te", "tes", "toi", "ton", "tu", "un", "une", "vos", "votre", "vous", "y",
];

/// Spanish stopwords (Snowball-derived set).
pub const SPANISH: &[&str] = &[
    "a", "al", "algo", "algunas", "alguno", "algunos", "ante", "antes", "como", "con", "contra",
    "cual", "cuando", "de", "del", "desde", "donde", "durante", "el", "ella", "ellas", "ellos",
    "en", "entre", "era", "esa", "esas", "ese", "eso", "esos", "esta", "estaba", "estado", "estas",
    "este", "esto", "estos", "fue", "ha", "hasta", "hay", "la", "las", "le", "les", "lo", "los",
    "mas", "me", "mi", "mio", "muy", "nada", "ni", "no", "nos", "nosotros", "nuestro", "o", "otra",
    "otras", "otro", "otros", "para", "pero", "por", "que", "quien", "se", "ser", "si", "sin",
    "sobre", "somos", "son", "soy", "su", "sus", "te", "ti", "tiene", "todo", "todos", "tu", "tus",
    "un", "una", "uno", "unos", "usted", "ustedes", "y", "ya", "yo",
];

/// Italian stopwords (Snowball-derived set).
pub const ITALIAN: &[&str] = &[
    "a", "abbia", "abbiamo", "abbiano", "ad", "agl", "agli", "ai", "al", "alla", "alle", "allo",
    "anche", "avere", "avesse", "avete", "aveva", "avevano", "c", "che", "chi", "ci", "come",
    "con", "contro", "cui", "da", "dagl", "dagli", "dai", "dal", "dall", "dalla", "dalle", "dallo",
    "degl", "degli", "dei", "del", "dell", "della", "delle", "dello", "di", "dov", "dove", "e",
    "era", "erano", "essere", "gli", "ha", "hai", "hanno", "ho", "i", "il", "in", "io", "l", "la",
    "le", "lei", "li", "lo", "loro", "lui", "ma", "me", "mi", "mia", "mie", "miei", "mio", "ne",
    "negl", "negli", "nei", "nel", "nell", "nella", "nelle", "nello", "noi", "non", "nostra",
    "nostre", "nostri", "nostro", "o", "per", "quale", "quanta", "quante", "quanti", "quanto",
    "quella", "quelle", "quelli", "quello", "questa", "queste", "questi", "questo", "sarebbe",
    "se", "si", "sia", "siamo", "sono", "sta", "stato", "sua", "sue", "sugl", "sugli", "sui",
    "sul", "sull", "sulla", "sulle", "sullo", "suo", "suoi", "ti", "tra", "tu", "tua", "tue",
    "tuo", "tuoi", "tutti", "tutto", "un", "una", "uno", "vi", "voi", "vostra", "vostre", "vostri",
    "vostro",
];

/// Portuguese stopwords (Snowball-derived set).
pub const PORTUGUESE: &[&str] = &[
    "a", "ao", "aos", "aquela", "aquelas", "aquele", "aqueles", "aquilo", "as", "ate", "com",
    "como", "da", "das", "de", "dela", "delas", "dele", "deles", "depois", "do", "dos", "e", "ela",
    "elas", "ele", "eles", "em", "entre", "era", "essa", "essas", "esse", "esses", "esta", "estas",
    "este", "estes", "eu", "foi", "foram", "ha", "isso", "isto", "ja", "lhe", "lhes", "lo", "mas",
    "me", "mesmo", "meu", "minha", "muito", "na", "nas", "no", "nos", "nossa", "nossas", "nosso",
    "nossos", "num", "numa", "o", "os", "ou", "para", "pela", "pelas", "pelo", "pelos", "por",
    "qual", "quando", "que", "quem", "sao", "se", "sem", "ser", "seu", "sua", "suas", "seus", "so",
    "tambem", "te", "tem", "ti", "tu", "tua", "tuas", "teu", "teus", "um", "uma", "uns", "umas",
    "voce", "voces", "vos",
];

/// Dutch stopwords (Snowball-derived set).
pub const DUTCH: &[&str] = &[
    "aan", "al", "alles", "als", "altijd", "andere", "ben", "bij", "daar", "dan", "dat", "de",
    "der", "deze", "die", "dit", "doch", "doen", "door", "dus", "een", "eens", "en", "er", "ge",
    "geen", "geweest", "haar", "had", "heb", "hebben", "heeft", "hem", "het", "hier", "hij", "hoe",
    "hun", "iemand", "iets", "ik", "in", "is", "ja", "je", "kan", "kon", "kunnen", "maar", "me",
    "meer", "men", "met", "mij", "mijn", "moet", "na", "naar", "niet", "niets", "nog", "nu", "of",
    "om", "omdat", "onder", "ons", "onze", "ook", "op", "over", "reeds", "te", "tegen", "toch",
    "toen", "tot", "u", "uit", "uw", "van", "veel", "voor", "want", "waren", "was", "wat", "we",
    "wel", "werd", "wij", "wil", "worden", "wordt", "zal", "ze", "zelf", "zich", "zij", "zijn",
    "zo", "zonder", "zou",
];

/// Russian stopwords (Snowball-derived set).
pub const RUSSIAN: &[&str] = &[
    "и",
    "в",
    "во",
    "не",
    "что",
    "он",
    "на",
    "я",
    "с",
    "со",
    "как",
    "а",
    "то",
    "все",
    "она",
    "так",
    "его",
    "но",
    "да",
    "ты",
    "к",
    "у",
    "же",
    "вы",
    "за",
    "бы",
    "по",
    "только",
    "ее",
    "мне",
    "было",
    "вот",
    "от",
    "меня",
    "еще",
    "нет",
    "о",
    "из",
    "ему",
    "теперь",
    "когда",
    "даже",
    "ну",
    "вдруг",
    "ли",
    "если",
    "уже",
    "или",
    "ни",
    "быть",
    "был",
    "него",
    "до",
    "вас",
    "нибудь",
    "опять",
    "уж",
    "вам",
    "ведь",
    "там",
    "потом",
    "себя",
    "ничего",
    "ей",
    "может",
    "они",
    "тут",
    "где",
    "есть",
    "надо",
    "ней",
    "для",
    "мы",
    "тебя",
    "их",
    "чем",
    "была",
    "сам",
    "чтоб",
    "без",
    "будто",
    "чего",
    "раз",
    "тоже",
    "себе",
    "под",
    "будет",
    "ж",
    "тогда",
    "кто",
    "этот",
    "того",
    "потому",
    "этого",
    "какой",
    "совсем",
    "ним",
    "здесь",
    "этом",
    "один",
    "почти",
    "мой",
    "тем",
    "чтобы",
    "нее",
    "сейчас",
    "были",
    "куда",
    "зачем",
    "всех",
    "никогда",
    "можно",
    "при",
    "наконец",
    "два",
    "об",
    "другой",
    "хоть",
    "после",
    "над",
    "больше",
    "тот",
    "через",
    "эти",
    "нас",
    "про",
    "всего",
    "них",
    "какая",
    "много",
    "разве",
    "три",
    "эту",
    "моя",
    "впрочем",
    "хорошо",
    "свою",
    "этой",
    "перед",
    "иногда",
    "лучше",
    "чуть",
    "том",
    "нельзя",
    "такой",
    "им",
    "более",
    "всегда",
    "конечно",
    "всю",
    "между",
];

/// Swedish stopwords (Snowball-derived set).
pub const SWEDISH: &[&str] = &[
    "alla", "allt", "att", "av", "blev", "bli", "blir", "blivit", "da", "dar", "de", "dem", "den",
    "denna", "deras", "dess", "dessa", "det", "detta", "dig", "din", "dina", "dit", "du", "efter",
    "ej", "eller", "en", "er", "era", "ert", "ett", "fran", "for", "ha", "hade", "han", "hans",
    "har", "har", "hon", "honom", "hur", "i", "icke", "ingen", "inom", "inte", "ja", "jag", "ju",
    "kan", "kunde", "man", "med", "mellan", "men", "mig", "min", "mina", "mitt", "mot", "mycket",
    "ni", "nagon", "nagot", "nagra", "nar", "nu", "och", "om", "oss", "pa", "sa", "samma", "sedan",
    "sin", "sina", "sist", "sitt", "ska", "skall", "skulle", "som", "till", "under", "upp", "ut",
    "utan", "vad", "var", "vara", "vart", "vi", "vid", "vid", "vill", "vi",
];

/// Norwegian stopwords (Snowball-derived set).
pub const NORWEGIAN: &[&str] = &[
    "alle", "at", "av", "bare", "begge", "ble", "blei", "bli", "blir", "blitt", "bort", "da", "de",
    "dei", "den", "denne", "der", "dere", "desse", "det", "dette", "di", "din", "disse", "du",
    "dykk", "eg", "ein", "eit", "eller", "en", "enn", "er", "et", "etter", "for", "fra", "ha",
    "hadde", "han", "hans", "har", "hennar", "henne", "hennes", "her", "ho", "hoe", "honom", "hun",
    "i", "ikkje", "ingen", "ingi", "inkje", "inn", "ja", "jeg", "kan", "kom", "korleis", "kun",
    "kunne", "kva", "kvar", "kvifor", "litt", "man", "mange", "med", "meg", "men", "mi", "min",
    "mine", "mitt", "mot", "mykje", "ned", "no", "noe", "noen", "noko", "nokon", "nokor", "noko",
    "nokre", "ny", "og", "om", "opp", "oss", "over", "pa", "sa", "same", "seg", "si", "sin",
    "sine", "sitt", "sist", "ska", "skal", "skulle", "so", "som", "somme", "somt", "til", "um",
    "under", "upp", "ut", "utan", "var", "vart", "vere", "vi", "vil", "ville", "viss", "vore",
    "vors",
];

/// Danish stopwords (Snowball-derived set).
pub const DANISH: &[&str] = &[
    "ad", "af", "aldrig", "alle", "alt", "anden", "andet", "andre", "at", "bare", "begge", "blev",
    "blive", "bliver", "da", "de", "dem", "den", "denne", "der", "deras", "deres", "det", "dette",
    "dig", "din", "dine", "disse", "dit", "dog", "du", "efter", "ej", "eller", "en", "end", "er",
    "et", "fleste", "for", "fordi", "fra", "ga", "gaa", "gaar", "gar", "gik", "gora", "gorde",
    "ham", "han", "hans", "har", "havde", "hen", "hende", "hendes", "her", "hos", "hun", "hvad",
    "hvem", "hver", "hvilken", "hvis", "hvor", "hvordan", "hvorfor", "hvornar", "i", "igen",
    "ikke", "ind", "ingen", "intet", "ja", "jeg", "jer", "jeres", "jo", "kan", "kom", "kun",
    "kunne", "lad", "langs", "lidt", "lige", "lille", "man", "mand", "mange", "med", "meget",
    "men", "mens", "mere", "mig", "min", "mine", "mit", "mod", "naar", "ned", "nej", "nogen",
    "noget", "nogle", "nu", "og", "op", "os", "over", "paa", "saa", "saadan", "se", "selv", "sin",
    "sine", "sit", "skal", "skulle", "som", "stor", "store", "synes", "thi", "til", "ud", "under",
    "var", "ved", "vi", "via", "vil", "ville", "vor", "vore", "vores", "vaere",
];

/// Finnish stopwords (Snowball-derived set).
pub const FINNISH: &[&str] = &[
    "ei", "ja", "jos", "kun", "me", "mutta", "niin", "ole", "on", "ovat", "se", "sita", "te",
    "tai", "tuo", "vaan", "han", "he", "itse", "joka", "jolla", "jonka", "jossa", "josta",
    "joiden", "joihin", "joilla", "joille", "joilta", "joissa", "joista", "joka", "jokin", "joku",
    "jolla", "jonka", "jopa", "jossa", "josta", "jota", "joten", "kanssa", "koko", "koska", "kuin",
    "kuka", "kun", "mika", "mille", "milla", "minka", "minkaan", "minun", "mista", "mita",
    "muiden", "muita", "mukainen", "mukaan", "mutta", "ne", "niin", "ole", "olla", "on", "ovat",
    "paitsi", "se", "seka", "siis", "silla", "sinun", "sita", "sitoi", "taa", "tai", "taikka",
    "tama", "te", "tuo", "vaan", "vai", "vaikka", "vain", "yli",
];

/// Hungarian stopwords (Snowball-derived set).
pub const HUNGARIAN: &[&str] = &[
    "a",
    "ahogy",
    "ahol",
    "aki",
    "akik",
    "akkor",
    "alalt",
    "altal",
    "altalaban",
    "amely",
    "amelyek",
    "amelyekben",
    "amelyeket",
    "amelyet",
    "amelynek",
    "ami",
    "amit",
    "amolyan",
    "amig",
    "amikor",
    "az",
    "azok",
    "azon",
    "azonban",
    "aztan",
    "azutan",
    "azzal",
    "be",
    "benne",
    "cikk",
    "cikkek",
    "cikkeket",
    "csak",
    "de",
    "e",
    "eddig",
    "egy",
    "egyes",
    "egyetlen",
    "egyeb",
    "egyik",
    "egyre",
    "ehhez",
    "el",
    "eleg",
    "ellen",
    "elso",
    "elott",
    "eloször",
    "emilyen",
    "en",
    "ennek",
    "erre",
    "ez",
    "ezek",
    "ezen",
    "ezt",
    "ezzel",
    "fel",
    "fele",
    "hanem",
    "hiszen",
    "hog",
    "hogy",
    "hogyan",
    "igen",
    "igy",
    "ill",
    "illetve",
    "inkabb",
    "is",
    "ison",
    "itt",
    "jobban",
    "jo",
    "jol",
    "kell",
    "kellett",
    "kerdes",
    "keresztul",
    "ki",
    "kicsit",
    "kozben",
    "kozott",
    "kozul",
    "kulonben",
    "kulonfele",
    "le",
    "lehet",
    "legyen",
    "lenni",
    "lenne",
    "lett",
    "maga",
    "magar",
    "majd",
    "mak",
    "mar",
    "mas",
    "masik",
    "meg",
    "megint",
    "mi",
    "miert",
    "mig",
    "mikor",
    "milyen",
    "mind",
    "mindegyik",
    "minden",
    "mindenki",
    "mindent",
    "mindig",
    "mint",
    "mintha",
    "mivel",
    "most",
    "nagy",
    "nagyobb",
    "nagyon",
    "ne",
    "neha",
    "nekem",
    "neki",
    "nem",
    "nincs",
    "ok",
    "ott",
    "pedig",
    "persze",
    "ra",
    "s",
    "sajat",
    "sem",
    "semmi",
    "sok",
    "sokat",
    "sokkal",
    "szamara",
    "szemben",
    "szerint",
    "szinte",
    "szoval",
    "talalt",
    "talan",
    "tavol",
    "te",
    "tehat",
    "teljes",
    "tovabb",
    "tovabba",
    "tul",
    "ugyan",
    "ugyanis",
    "uj",
    "ujabb",
    "ujra",
    "utan",
    "utana",
    "utolso",
    "vagy",
    "vagyis",
    "valaki",
    "valami",
    "valamint",
    "valo",
    "viszont",
    "volt",
    "volna",
    "ahol",
];

/// Romanian stopwords (Snowball-derived set).
pub const ROMANIAN: &[&str] = &[
    "a",
    "abia",
    "aceasta",
    "aceast",
    "aceea",
    "acei",
    "aceia",
    "acel",
    "acela",
    "acelasi",
    "acele",
    "acelea",
    "acest",
    "acesta",
    "aceste",
    "acestea",
    "acestei",
    "acestia",
    "aci",
    "acolo",
    "acord",
    "acum",
    "ai",
    "aia",
    "aiba",
    "aici",
    "al",
    "ala",
    "alea",
    "alt",
    "alta",
    "alte",
    "altfel",
    "altul",
    "am",
    "anume",
    "apoi",
    "ar",
    "are",
    "asa",
    "asemenea",
    "asta",
    "astazi",
    "astel",
    "astfei",
    "asupra",
    "atare",
    "atat",
    "atata",
    "atatea",
    "atatia",
    "ati",
    "atit",
    "atita",
    "atitea",
    "atitia",
    "au",
    "avea",
    "avem",
    "aveti",
    "azi",
    "ba",
    "bine",
    "bucur",
    "ca",
    "cam",
    "cand",
    "capat",
    "care",
    "careia",
    "carora",
    "caruia",
    "cat",
    "cata",
    "cate",
    "cateodata",
    "cativa",
    "catre",
    "ce",
    "cel",
    "ceilalti",
    "ceva",
    "chiar",
    "ci",
    "cine",
    "cineva",
    "cit",
    "cita",
    "cite",
    "citeva",
    "citiva",
    "conform",
    "contra",
    "cu",
    "cum",
    "cumva",
    "da",
    "daca",
    "dar",
    "dat",
    "datorita",
    "de",
    "deasupra",
    "deci",
    "decit",
    "deja",
    "deoarece",
    "departe",
    "desi",
    "despre",
    "destul",
    "din",
    "dinaintea",
    "dintre",
    "dupa",
    "e",
    "ea",
    "ei",
    "el",
    "ele",
    "era",
    "erau",
    "este",
    "eu",
    "exact",
    "exista",
    "fi",
    "fie",
    "fiecare",
    "fii",
    "fim",
    "fiti",
    "fost",
    "i",
    "ia",
    "iar",
    "ieri",
    "il",
    "imi",
    "in",
    "inainte",
    "inaintea",
    "incit",
    "insa",
    "intre",
    "isi",
    "iti",
    "la",
    "le",
    "li",
    "lor",
    "lui",
    "ma",
    "mai",
    "mare",
    "mea",
    "mei",
    "meu",
    "mi",
    "mie",
    "mine",
    "mod",
    "moi",
    "mult",
    "multa",
    "multe",
    "multi",
    "multumesc",
    "ne",
    "neatins",
    "ni",
    "nici",
    "niciodata",
    "nimeni",
    "nimic",
    "niste",
    "noi",
    "nor",
    "nostru",
    "nu",
    "o",
    "oare",
    "odata",
    "ori",
    "orice",
    "pe",
    "pana",
    "pentru",
    "peste",
    "pina",
    "poate",
    "pot",
    "prea",
    "prima",
    "prin",
    "printre",
    "sa",
    "sau",
    "se",
    "si",
    "sintem",
    "sub",
    "sunt",
    "suntem",
    "ta",
    "tale",
    "te",
    "ti",
    "tine",
    "totusi",
    "tu",
    "tuturor",
    "un",
    "una",
    "unde",
    "undeva",
    "uneia",
    "unele",
    "uneori",
    "unii",
    "unor",
    "unora",
    "unul",
    "va",
    "vi",
    "voi",
    "voua",
    "vreo",
    "vreun",
];

/// Turkish stopwords (Snowball-derived set).
pub const TURKISH: &[&str] = &[
    "acaba",
    "altmis",
    "alti",
    "ama",
    "ancak",
    "arada",
    "aslinda",
    "ayni",
    "bana",
    "bazi",
    "belki",
    "ben",
    "benden",
    "beni",
    "benim",
    "beri",
    "bes",
    "bile",
    "bin",
    "bir",
    "biraz",
    "bircok",
    "biri",
    "birisi",
    "birkac",
    "biz",
    "bize",
    "bizden",
    "bizi",
    "bizim",
    "boyle",
    "bu",
    "buna",
    "bunda",
    "bundan",
    "bunlar",
    "bunlari",
    "bunlarin",
    "bunu",
    "bunun",
    "burada",
    "buyuk",
    "cok",
    "cunku",
    "da",
    "daha",
    "dahi",
    "de",
    "defa",
    "degil",
    "diger",
    "diye",
    "doksan",
    "dokuz",
    "dolayi",
    "dolayisiyla",
    "dort",
    "durumu",
    "edecek",
    "eden",
    "ederek",
    "edilecek",
    "ediliyor",
    "edilmesi",
    "edi",
    "elli",
    "en",
    "etmesi",
    "etti",
    "ettigi",
    "ettigini",
    "gibi",
    "gore",
    "hala",
    "halde",
    "hep",
    "hepsi",
    "her",
    "herhangi",
    "herkes",
    "hic",
    "hicbir",
    "icin",
    "iki",
    "ile",
    "ilgili",
    "ise",
    "iste",
    "itibaren",
    "iyi",
    "karsin",
    "kendi",
    "kendine",
    "kendini",
    "ki",
    "kim",
    "kime",
    "kimi",
    "kimse",
    "kirk",
    "milyar",
    "milyon",
    "mu",
    "nasil",
    "ne",
    "neden",
    "nedenle",
    "nerde",
    "nerede",
    "nereye",
    "niye",
    "o",
    "olan",
    "olarak",
    "oldu",
    "oldugu",
    "oldugunu",
    "olduklarini",
    "olmadi",
    "olmak",
    "olmasi",
    "olmayan",
    "olsa",
    "olup",
    "olur",
    "olursa",
    "oluyor",
    "on",
    "ona",
    "ondan",
    "onlar",
    "onlari",
    "onlarin",
    "onu",
    "onun",
    "orada",
    "otuz",
    "oysa",
    "pek",
    "ramen",
    "sana",
    "sekiz",
    "seksen",
    "sen",
    "senden",
    "seni",
    "senin",
    "siz",
    "sizden",
    "sizi",
    "sizin",
    "su",
    "suna",
    "sunu",
    "sunun",
    "tarafindan",
    "trilyon",
    "tum",
    "ve",
    "veya",
    "ya",
    "yani",
    "yapacak",
    "yapilan",
    "yapilmasi",
    "yapmak",
    "yaptigi",
    "yaptiklari",
    "yedi",
    "yetmis",
    "yine",
    "yirmi",
    "yoksa",
    "yuz",
    "zaten",
];

/// Arabic stopwords (common set derived from standard NLP resources).
pub const ARABIC: &[&str] = &[
    "في",
    "من",
    "على",
    "إلى",
    "عن",
    "هذا",
    "هذه",
    "التي",
    "الذي",
    "التى",
    "هو",
    "هي",
    "كان",
    "كانت",
    "لا",
    "ما",
    "لم",
    "لن",
    "مع",
    "أو",
    "و",
    "ان",
    "أن",
    "بعد",
    "قبل",
    "كل",
    "ذلك",
    "تلك",
    "هل",
    "ثم",
    "بين",
    "حتى",
    "إذا",
    "كما",
    "لكن",
    "بل",
    "منذ",
    "عند",
    "قد",
    "لقد",
    "غير",
    "ولا",
    "فقد",
    "أي",
    "فإن",
    "أنه",
    "إنه",
    "نحن",
    "هم",
    "أنت",
    "هنا",
    "هناك",
    "كيف",
    "لماذا",
    "أين",
    "متى",
];

/// Hindi stopwords (common set derived from standard NLP resources).
pub const HINDI: &[&str] = &[
    "का",
    "के",
    "की",
    "में",
    "है",
    "हैं",
    "से",
    "को",
    "पर",
    "ने",
    "और",
    "एक",
    "यह",
    "वह",
    "हो",
    "था",
    "थे",
    "थी",
    "जो",
    "कि",
    "इस",
    "उस",
    "नहीं",
    "कर",
    "भी",
    "तो",
    "ही",
    "या",
    "अपने",
    "लिए",
    "कुछ",
    "जब",
    "तक",
    "सब",
    "अभी",
    "होता",
    "बहुत",
    "दिया",
    "कोई",
    "अगर",
    "वे",
    "हम",
    "तुम",
    "मैं",
    "किया",
    "गया",
    "इसके",
    "उसके",
    "सकता",
    "साथ",
    "जैसे",
    "बाद",
    "पहले",
    "ऐसे",
    "सभी",
    "रहा",
    "रहे",
    "अपना",
    "अपनी",
    "दो",
    "वाले",
    "होती",
    "होते",
    "बस",
];

/// Return the English stopword list as owned strings.
pub fn english() -> Vec<String> {
    ENGLISH.iter().map(|s| (*s).to_string()).collect()
}

/// Return the German stopword list as owned strings.
pub fn german() -> Vec<String> {
    GERMAN.iter().map(|s| (*s).to_string()).collect()
}

/// Return the French stopword list as owned strings.
pub fn french() -> Vec<String> {
    FRENCH.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Spanish stopword list as owned strings.
pub fn spanish() -> Vec<String> {
    SPANISH.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Italian stopword list as owned strings.
pub fn italian() -> Vec<String> {
    ITALIAN.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Portuguese stopword list as owned strings.
pub fn portuguese() -> Vec<String> {
    PORTUGUESE.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Dutch stopword list as owned strings.
pub fn dutch() -> Vec<String> {
    DUTCH.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Russian stopword list as owned strings.
pub fn russian() -> Vec<String> {
    RUSSIAN.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Swedish stopword list as owned strings.
pub fn swedish() -> Vec<String> {
    SWEDISH.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Norwegian stopword list as owned strings.
pub fn norwegian() -> Vec<String> {
    NORWEGIAN.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Danish stopword list as owned strings.
pub fn danish() -> Vec<String> {
    DANISH.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Finnish stopword list as owned strings.
pub fn finnish() -> Vec<String> {
    FINNISH.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Hungarian stopword list as owned strings.
pub fn hungarian() -> Vec<String> {
    HUNGARIAN.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Romanian stopword list as owned strings.
pub fn romanian() -> Vec<String> {
    ROMANIAN.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Turkish stopword list as owned strings.
pub fn turkish() -> Vec<String> {
    TURKISH.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Arabic stopword list as owned strings.
pub fn arabic() -> Vec<String> {
    ARABIC.iter().map(|s| (*s).to_string()).collect()
}

/// Return the Hindi stopword list as owned strings.
pub fn hindi() -> Vec<String> {
    HINDI.iter().map(|s| (*s).to_string()).collect()
}

/// Return stopwords for a given language name (case-insensitive).
///
/// Returns `None` if the language is not supported.
pub fn for_language(language: &str) -> Option<Vec<String>> {
    match language.to_lowercase().as_str() {
        "english" | "en" => Some(english()),
        "german" | "de" => Some(german()),
        "french" | "fr" => Some(french()),
        "spanish" | "es" => Some(spanish()),
        "italian" | "it" => Some(italian()),
        "portuguese" | "pt" => Some(portuguese()),
        "dutch" | "nl" => Some(dutch()),
        "russian" | "ru" => Some(russian()),
        "swedish" | "sv" => Some(swedish()),
        "norwegian" | "no" => Some(norwegian()),
        "danish" | "da" => Some(danish()),
        "finnish" | "fi" => Some(finnish()),
        "hungarian" | "hu" => Some(hungarian()),
        "romanian" | "ro" => Some(romanian()),
        "turkish" | "tr" => Some(turkish()),
        "arabic" | "ar" => Some(arabic()),
        "hindi" | "hi" => Some(hindi()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_stopwords_not_empty() {
        assert!(!ENGLISH.is_empty());
        assert!(ENGLISH.contains(&"the"));
    }

    #[test]
    fn german_stopwords_not_empty() {
        assert!(!GERMAN.is_empty());
        assert!(GERMAN.contains(&"der"));
    }

    #[test]
    fn french_stopwords_not_empty() {
        assert!(!FRENCH.is_empty());
        assert!(FRENCH.contains(&"le"));
    }

    #[test]
    fn spanish_stopwords_not_empty() {
        assert!(!SPANISH.is_empty());
        assert!(SPANISH.contains(&"el"));
    }

    #[test]
    fn italian_stopwords_not_empty() {
        assert!(!ITALIAN.is_empty());
        assert!(ITALIAN.contains(&"il"));
    }

    #[test]
    fn portuguese_stopwords_not_empty() {
        assert!(!PORTUGUESE.is_empty());
        assert!(PORTUGUESE.contains(&"de"));
    }

    #[test]
    fn dutch_stopwords_not_empty() {
        assert!(!DUTCH.is_empty());
        assert!(DUTCH.contains(&"de"));
    }

    #[test]
    fn russian_stopwords_not_empty() {
        assert!(!RUSSIAN.is_empty());
        assert!(RUSSIAN.contains(&"и"));
    }

    #[test]
    fn swedish_stopwords_not_empty() {
        assert!(!SWEDISH.is_empty());
        assert!(SWEDISH.contains(&"och"));
    }

    #[test]
    fn norwegian_stopwords_not_empty() {
        assert!(!NORWEGIAN.is_empty());
        assert!(NORWEGIAN.contains(&"og"));
    }

    #[test]
    fn danish_stopwords_not_empty() {
        assert!(!DANISH.is_empty());
        assert!(DANISH.contains(&"og"));
    }

    #[test]
    fn finnish_stopwords_not_empty() {
        assert!(!FINNISH.is_empty());
        assert!(FINNISH.contains(&"ja"));
    }

    #[test]
    fn hungarian_stopwords_not_empty() {
        assert!(!HUNGARIAN.is_empty());
        assert!(HUNGARIAN.contains(&"a"));
    }

    #[test]
    fn romanian_stopwords_not_empty() {
        assert!(!ROMANIAN.is_empty());
        assert!(ROMANIAN.contains(&"de"));
    }

    #[test]
    fn turkish_stopwords_not_empty() {
        assert!(!TURKISH.is_empty());
        assert!(TURKISH.contains(&"bir"));
    }

    #[test]
    fn arabic_stopwords_not_empty() {
        assert!(!ARABIC.is_empty());
        assert!(ARABIC.contains(&"في"));
    }

    #[test]
    fn hindi_stopwords_not_empty() {
        assert!(!HINDI.is_empty());
        assert!(HINDI.contains(&"है"));
    }

    #[test]
    fn for_language_lookup() {
        assert!(for_language("english").is_some());
        assert!(for_language("en").is_some());
        assert!(for_language("German").is_some());
        assert!(for_language("DE").is_some());
        assert!(for_language("unknown_lang").is_none());
    }

    #[test]
    fn all_17_languages_accessible() {
        let langs = [
            "english",
            "german",
            "french",
            "spanish",
            "italian",
            "portuguese",
            "dutch",
            "russian",
            "swedish",
            "norwegian",
            "danish",
            "finnish",
            "hungarian",
            "romanian",
            "turkish",
            "arabic",
            "hindi",
        ];
        for lang in &langs {
            let words = for_language(lang);
            assert!(words.is_some(), "missing stopwords for {lang}");
            assert!(
                !words.as_ref().unwrap().is_empty(),
                "empty stopwords for {lang}"
            );
        }
    }

    #[test]
    fn all_17_languages_have_canonical_words() {
        // Verify each language list contains at least one expected canonical stopword.
        let canonical: &[(&str, &str)] = &[
            ("english", "the"),
            ("german", "der"),
            ("french", "le"),
            ("spanish", "el"),
            ("italian", "il"),
            ("portuguese", "de"),
            ("dutch", "de"),
            ("russian", "\u{0438}"), // "и"
            ("swedish", "och"),
            ("norwegian", "og"),
            ("danish", "og"),
            ("finnish", "ja"),
            ("hungarian", "a"),
            ("romanian", "de"),
            ("turkish", "bir"),
            ("arabic", "\u{0641}\u{064a}"), // "في"
            ("hindi", "\u{0939}\u{0948}"),  // "है"
        ];
        for &(lang, word) in canonical {
            let words =
                for_language(lang).unwrap_or_else(|| panic!("missing stopwords for {lang}"));
            assert!(
                words.contains(&word.to_string()),
                "{lang}: expected canonical word '{word}' not found in stopword list"
            );
        }
    }

    #[test]
    fn for_language_iso_code_lookup_all() {
        let codes = [
            "en", "de", "fr", "es", "it", "pt", "nl", "ru", "sv", "no", "da", "fi", "hu", "ro",
            "tr", "ar", "hi",
        ];
        for code in &codes {
            assert!(
                for_language(code).is_some(),
                "ISO code '{code}' should return stopwords"
            );
        }
    }

    #[test]
    fn for_language_case_insensitive() {
        assert!(for_language("ENGLISH").is_some());
        assert!(for_language("English").is_some());
        assert!(for_language("eNgLiSh").is_some());
    }

    #[test]
    fn stopword_lists_contain_no_empty_strings() {
        let all_lists: &[(&str, &[&str])] = &[
            ("english", ENGLISH),
            ("german", GERMAN),
            ("french", FRENCH),
            ("spanish", SPANISH),
            ("italian", ITALIAN),
            ("portuguese", PORTUGUESE),
            ("dutch", DUTCH),
            ("russian", RUSSIAN),
            ("swedish", SWEDISH),
            ("norwegian", NORWEGIAN),
            ("danish", DANISH),
            ("finnish", FINNISH),
            ("hungarian", HUNGARIAN),
            ("romanian", ROMANIAN),
            ("turkish", TURKISH),
            ("arabic", ARABIC),
            ("hindi", HINDI),
        ];
        for &(lang, list) in all_lists {
            for word in list {
                assert!(
                    !word.is_empty(),
                    "{lang}: stopword list contains an empty string"
                );
            }
        }
    }

    #[test]
    fn owned_accessor_returns_same_count_as_const() {
        assert_eq!(english().len(), ENGLISH.len());
        assert_eq!(german().len(), GERMAN.len());
        assert_eq!(french().len(), FRENCH.len());
        assert_eq!(spanish().len(), SPANISH.len());
        assert_eq!(italian().len(), ITALIAN.len());
        assert_eq!(portuguese().len(), PORTUGUESE.len());
        assert_eq!(dutch().len(), DUTCH.len());
        assert_eq!(russian().len(), RUSSIAN.len());
        assert_eq!(swedish().len(), SWEDISH.len());
        assert_eq!(norwegian().len(), NORWEGIAN.len());
        assert_eq!(danish().len(), DANISH.len());
        assert_eq!(finnish().len(), FINNISH.len());
        assert_eq!(hungarian().len(), HUNGARIAN.len());
        assert_eq!(romanian().len(), ROMANIAN.len());
        assert_eq!(turkish().len(), TURKISH.len());
        assert_eq!(arabic().len(), ARABIC.len());
        assert_eq!(hindi().len(), HINDI.len());
    }
}
