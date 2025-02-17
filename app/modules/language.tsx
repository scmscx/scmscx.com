import { Accessor, Signal } from "solid-js";
import { useLang } from "./context";

interface LangMap {
  [key: string]: LangMap2;
}
interface LangMap2 {
  [key: string]: string;
}

const langmap = {
  ru: {
    // Navbar
    Home: "Главная",
    Search: "Поиск",
    Upload: "Загрузка",
    About: "О проекте",
    "Log in": "Войти",

    // Home Page
    "Welcome to scmscx.com": "Добро пожаловать на scmscx.com",
    "The largest StarCraft: Brood War map database in the universe":
      "Крупнейшая база карт для StarCraft: Brood War во Вселенной",

    "Recently Viewed Maps": "Недавно просмотренные карты",
    "Recently Downloaded Maps": "Недавно скачанные карты",
    "Recently Uploaded Maps": "Недавно закачанные карты",
    "Featured Maps": "Популярные карты",

    // Search
    Query: "Поиск",
    Random: "Случайная карта",

    Targets: "Цели",
    Filters: "Фильтры",
    Sorting: "Сортировка",
    Results: "Результаты",

    Scenario: "Сценарии",
    "Last Modified": "Последнее изменение",
    "Time Uploaded": "Время загрузки",

    Units: "Единицы",
    Forces: "Силы",
    Filenames: "Названия",
    Scenarios: "Сценарии",
    "Scenario Descriptions": "Описания сценариев",

    "Minimum Map Width": "Минимальная ширина карты",
    "Maximum Map Width": "Максимальная ширина карты",

    "Minimum Map Height": "Минимальная высота карты",
    "Maximum Map Height": "Максимальная высота карты",

    "Minimum Human Players": "Минимальное число игроков-людей",
    "Maximum Human Players": "Максимальное число игроков-людей",

    "Minimum Computer Players": "Минимальное число компьютерных игроков",
    "Maximum Computer Players": "Максимальное число компьютерных игроков",

    "Last Modified After": "Последнее изменение после",
    "Last Modified Before": "Последнее изменение до",
    "Time Uploaded After": "Время загрузки после",
    "Time Uploaded Before": "Время загрузки до",

    Badlands: "Пустошь",
    Space: "Космос",
    Installation: "Станция",
    Ashworld: "Вулкан",
    Jungle: "Джунгли",
    Desert: "Пустыня",
    Ice: "Лёд",
    Twilight: "Сумрак",

    Relevancy: "Релевантность",
    "Last Modified (Oldest First)":
      "Последнее изменение (сначала самая старая)",
    "Last Modified (Newest First)": "Последнее изменение (сначала самая новая)",
    "Time Uploaded (Oldest First)": "Время загрузки (сначала самая старая)",
    "Time Uploaded (Newest First)": "Время загрузки (сначала самая новая)",

    // Map page
    Download: "Скачать",
    Minimap: "Мини-карта",

    "Scenario Properties": "Свойства сценария",
    Version: "Версия",
    Tileset: "Местность",
    Dimensions: "Размеры",
    Locations: "Локации",
    Doodads: "Декорации",
    Sprites: "Спрайты",
    Triggers: "Триггеры",
    "Briefing Triggers": "Триггеры брифинга",

    Replays: "Повторы",
    Duration: "Продолжительность",
    "Time Recorded": "Время записи",
    Creator: "Создатель",

    "Known Filenames": "Известные имена файлов",
    Filename: "Имя файла",

    "Known Timestamps": "Известные даты",
    "Last Modified Time": "Время последнего изменения",

    Unit: "Единица",
    Name: "Название",

    "Similar Maps": "Похожие карты",

    Flags: "Флаги",
    Unfinished: "Незавершённая",
    Outdated: "Устаревшая",
    Broken: "Сломанная",
    "Black Holed": "Чёрная дыра",
    "Spoiler Unit Names": "Названия споилированных единиц",

    Key: "Ключ",
    Value: "Значение",

    Tags: "Теги",

    "MPQ Hash": "Хэш MPQ",
    "MPQ Size": "Размер MPQ",
    "CHK Hash": "Хэш CHK",
    "CHK Size": "Размер CHK",
    "Uploaded by": "Кем загружено",
    "Uploaded On": "Когда загружено",
    "Last Viewed": "Последний просмотр",
    "Last Downloaded": "Последнее скачивание",
    Views: "Просмотров",
    Downloads: "Скачиваний",

    // Upload
    "If you want to upload one or more .scm/.scx files, then choose the top file picker.":
      "Если вы хотите загрузить один или несколько файлов .scm/.scx, выберите верхний вариант выбора файлов.",

    "If you want to upload entire directories and their sub directories, then choose the bottom file picker.":
      "Если вы хотите загрузить весь каталог и его подкаталоги, выберите нижний вариант выбора файлов.",

    "Don't worry about corrupt, broken, unfinished, testing, duplicate, or outdated maps. The website will handle all of this and many of them are important parts of StarCraft map making history. Even uploading the exact same maps multiple times is no concern. So, upload everything you have and let the site do the filtering and processing.":
      "Не беспокойтесь о повреждённых, сломанных, незавершённых, тестируемых, дублирующихся или устаревших картах. Веб-сайт будет обрабатывать всё это, а многие из них являются важной частью истории создания карт для StarCraft. Даже загрузка одних и тех же карт несколько раз - не проблема. Итак, загрузите всё, что у вас есть, и позвольте сайту выполнить фильтрацию и обработку.",

    "Try uploading your entire StarCraft map directory, it can commonly be found at:":
      "Попробуйте загрузить весь каталог StarCraft. Обычно он находится по адресу:",

    ".scm/.scx file upload": "Загрузка файлов .scm/.scx",

    "Directory (and sub directories) upload":
      "Загрузка каталога (и его подкаталогов)",
    "Uploads are disabled while maintenance is being performed. Expected duration 2 hours.":
      "Загрузки отключены, пока проводятся работы. Предполагаемая продолжительность 2 часа.",

    "In Progress": "В процессе",
    Progress: "Прогресс",
    Size: "Размер",
    Pending: "В очереди",
    Failed: "Не удалось",
    "Retry All": "Повторить все",
    Retry: "Повторить",
    Reason: "Причина",
    Completed: "Завершено",
    Link: "Ссылка",

    // About Page
    "Frequently Asked Questions": "Часто задаваемые вопросы",
    "What is this website?": "Что это за сайт?",
    "Why does this website exist?": "Почему этот сайт существует?",
    "What does scmscx.com mean?": "Что значит scmscx.com?",
    "Why do we need another map database website?":
      "Зачем нам нужен ещё один веб-сайт с базой данных карт?",

    "How do I play the maps after I have downloaded from here?":
      "Как играть карты после загрузки с этого сайта?",

    "How many maps does the database have?": "Сколько карт в базе данных?",

    "Can I contribute maps to the database?":
      "Могу ли я участвовать в пополнении базы данных карт?",

    "How is this website made? What is the technology behind it?":
      "Как этот сайт сделан? Какая технология за ним стоит?",

    "Can I link directly to map downloads or minimap previews?":
      "Могу ли я ссылаться напрямую на загрузки карт или миниатюры карт?",

    "I found a bug. How do I report it?":
      "Я нашел ошибку. Как я могу ее сообщить?",

    "How can I contact you?": "Как я могу связаться с вами?",

    "Can I make a donation, how much does this site cost to run?":
      "Могу ли я сделать пожертвование, во сколько обходится работа этого сайта?",

    "Credit and Thanks": "Титры и благодарности",

    Devlog: "журнал разработки",

    // User page
    "Log out": "Выход",
    "Change Username": "Изменить имя пользователя",
    "New Username": "Новое имя пользователя",
    "Confirm New Username": "Подтвердите новое имя пользователя",
    "Current Password": "Текущий пароль",
    "Change Password": "Изменить пароль",
    "New Password": "Новый пароль",
    "Confirm New Password": "Подтвердите новый пароль",

    // Log In page
    Username: "Имя пользователя",
    Password: "Пароль",
    Register: "Регистрация",
    "Confirm Password": "Подтвердите пароль",
  },
  es: {
    // Navbar
    Home: "Inicio",
    Search: "Buscar",
    Upload: "Subir",
    About: "Acerca de",
    "Log in": "Iniciar sesión",

    // Home Page
    "Welcome to scmscx.com": "Bienvenido a scmscx.com",
    "The largest StarCraft: Brood War map database in the universe":
      "La mayor base de datos de mapas de StarCraft: Brood War del universo",

    "Recently Viewed Maps": "Mapas vistos recientemente",
    "Recently Downloaded Maps": "Mapas descargados recientemente",
    "Recently Uploaded Maps": "Mapas subidos recientemente",
    "Featured Maps": "Mapas destacados",

    // Search
    Query: "Busque",
    Random: "Aleatorio",

    Targets: "Objetivos",
    Filters: "Filtros",
    Sorting: "Ordenamiento",
    Results: "Resultados",

    Scenario: "Escenario",
    "Last Modified": "Última modificación",
    "Time Uploaded": "Fecha de subida",

    Units: "Unidades",
    Forces: "Fuerzas",
    Filenames: "Nombres de archivo",
    Scenarios: "Escenarios",
    "Scenario Descriptions": "Descripción del escenario",

    "Minimum Map Width": "Anchura mínima del mapa",
    "Maximum Map Width": "Anchura máxima del mapa",

    "Minimum Map Height": "Altura mínima del mapa",
    "Maximum Map Height": "Altura máxima del mapa",

    "Minimum Human Players": "Jugadores humanos mínimos",
    "Maximum Human Players": "Jugadores humanos máximos",

    "Minimum Computer Players": "Jugadores de computadora mínimos",
    "Maximum Computer Players": "Jugadores de computadora máximos",

    "Last Modified After": "Última modificación despues de",
    "Last Modified Before": "Última modificación antes de",
    "Time Uploaded After": "fecha de subida despues de",
    "Time Uploaded Before": "fecha de subida antes de",

    Badlands: "Tierras áridas",
    Space: "Espacio",
    Installation: "Instalación",
    Ashworld: "Planeta de ceniza",
    Jungle: "Jungla",
    Desert: "Desierto",
    Ice: "Hielo",
    Twilight: "Crepúsculo",

    Relevancy: "Relevancia",
    "Last Modified (Oldest First)":
      "Última modificación (la más antigua primero)",
    "Last Modified (Newest First)":
      "Última modificación (la más reciente primero)",
    "Time Uploaded (Oldest First)": "fecha de subida (primero la más antigua)",
    "Time Uploaded (Newest First)": "fecha de subida (la más reciente primero)",

    // Map page
    Download: "Descargar",
    Minimap: "Minimapa",

    "Scenario Properties": "Propiedades del escenario",
    Version: "Versión",
    Tileset: "Set de teselas",
    Dimensions: "Dimensiones",
    Locations: "Ubicaciones",
    Doodads: "Doodads",
    Sprites: "Sprites",
    Triggers: "Disparadores",
    "Briefing Triggers": "Disparadores de briefing",

    Replays: "Replays",
    Duration: "Duración",
    "Time Recorded": "Tiempo registrado",
    Creator: "Creador",

    "Known Filenames": "Nombres de archivo conocidos",
    Filename: "Nombre de archivo",

    "Known Timestamps": "Marcas de tiempo conocidas",
    "Last Modified Time": "Última modificación",

    Unit: "Unidad",
    Name: "Nombre",

    "Similar Maps": "Mapas similares",

    Flags: "Indicadores",
    Unfinished: "Sin terminar",
    Outdated: "Desactualizado",
    Broken: "Roto",
    "Black Holed": "Arrojado a un agujero negro",
    "Spoiler Unit Names": "Nombres de unidades de spoiler",

    Key: "Clave",
    Value: "Valor",

    Tags: "Etiquetas",

    "MPQ Hash": "Hash de MPQ",
    "MPQ Size": "Talla de MPQ",
    "CHK Hash": "Hash de CHK",
    "CHK Size": "Talla de CHK",
    "Uploaded by": "Subido por",
    "Uploaded On": "Subido en",
    "Last Viewed": "Última vista",
    "Last Downloaded": "Última descarga",
    Views: "Vistas",
    Downloads: "Descargas",

    // Upload
    "If you want to upload one or more .scm/.scx files, then choose the top file picker.":
      "Si desea cargar uno o varios archivos .scm/.scx, elija el selector de archivos superior.",

    "If you want to upload entire directories and their sub directories, then choose the bottom file picker.":
      "Si desea cargar directorios completos y sus subdirectorios, elija el selector de archivos inferior.",

    "Don't worry about corrupt, broken, unfinished, testing, duplicate, or outdated maps. The website will handle all of this and many of them are important parts of StarCraft map making history. Even uploading the exact same maps multiple times is no concern. So, upload everything you have and let the site do the filtering and processing.":
      "No te preocupes por mapas corruptos, rotos, inacabados, en pruebas, duplicados o desactualizados. El sitio web se encargará de todo esto y muchos de ellos son partes importantes de la historia de la creación de mapas de StarCraft. Incluso subir exactamente los mismos mapas varias veces no es un problema. Así que sube todo lo que tengas y deja que el sitio se encargue de filtrarlo y procesarlo.",

    "Try uploading your entire StarCraft map directory, it can commonly be found at:":
      "Intenta subir todo tu directorio de mapas de StarCraft, normalmente se encuentra en:",

    ".scm/.scx file upload": "Carga de archivos .scm/.scx",

    "Directory (and sub directories) upload":
      "Carga de directorios (y subdirectorios)",
    "Uploads are disabled while maintenance is being performed. Expected duration 2 hours.":
      "Las subidas estan desactivadas mientras se realiza mantenimiento. Duración estimada de 2 horas.",

    "In Progress": "En progreso",
    Progress: "Progreso",
    Size: "Talla",
    Pending: "Pendiente",
    Failed: "Fallido",
    "Retry All": "Reintentar todo",
    Retry: "Reintentar",
    Reason: "Razón",
    Completed: "Completado",
    Link: "Enlace",

    // About Page
    "Frequently Asked Questions": "Preguntas frecuentes",
    "What is this website?": "¿Qué es este sitio web?",
    "Why does this website exist?": "¿Por qué existe este sitio web?",
    "What does scmscx.com mean?": "¿Qué significa scmscx.com?",
    "Why do we need another map database website?":
      "¿Por qué necesitamos otro sitio web de bases de de mapas?",

    "How do I play the maps after I have downloaded from here?":
      "¿Cómo puedo jugar los mapas después de descargarlos desde aquí?",

    "How many maps does the database have?":
      "¿Cuántos mapas tiene la base de datos?",

    "Can I contribute maps to the database?":
      "¿Puedo aportar mapas a la base de datos?",

    "How is this website made? What is the technology behind it?":
      "¿Cómo está hecho este sitio web? ¿Qué tecnología utiliza?",

    "Can I link directly to map downloads or minimap previews?":
      "¿Puedo enlazar directamente con descargas de mapas o vistas previas de minimapas?",

    "I found a bug. How do I report it?":
      "¿Encontre un error. Como puedo reportarlo?",

    "How can I contact you?": "¿Cómo puedo ponerme en contacto con usted?",

    "Can I make a donation, how much does this site cost to run?":
      "¿Puedo hacer una donación? ¿Cuánto cuesta el funcionamiento de este sitio?",

    "Credit and Thanks": "Créditos y agradecimientos",

    Devlog: "Registro de desarrollo",

    // User page
    "Log out": "Cerrar sesión",
    "Change Username": "Cambiar nombre de usuario",
    "New Username": "Nuevo nombre de usuario",
    "Confirm New Username": "Confirmar nuevo nombre de usuario",
    "Current Password": "Contraseña actual",
    "Change Password": "Cambiar contraseña",
    "New Password": "Nueva contraseña",
    "Confirm New Password": "Confirmar nueva contraseña",

    // Log In page
    Username: "Nombre de usuario",
    Password: "Contraseña",
    Register: "Registrar",
    "Confirm Password": "Confirmar contraseña",
  },
  ko: {
    // Navbar
    Home: "홈으로",
    Search: "검색",
    Recent: "최신순",
    About: "정보",
    Upload: "업로드",
    "Upload Map": "맵 업로드",
    "Upload Replay": "다시보기 업로드",
    "Change Username": "사용자 이름 변경",
    "Change Password": "비밀번호 변경",
    "Log in": "로그인",
    "Log out": "로그아웃",

    // Upload
    "If you want to upload one or more .scm/.scx files, then choose the top file picker.":
      ".scm/.scx 파일 하나 또는 여러 개를 업로드하려면 위쪽의 파일 선택 기능을 사용하세요.",

    "If you want to upload entire directories and their sub directories, then choose the bottom file picker.":
      "디렉터리 전체 및 하위 디렉터리까지 업로드하려면 아래쪽의 파일 선택 기능을 사용하세요.",

    "Don't worry about corrupt, broken, unfinished, testing, duplicate, or outdated maps. The website will handle all of this and many of them are important parts of StarCraft map making history. Even uploading the exact same maps multiple times is no concern. So, upload everything you have and let the site do the filtering and processing.":
      "손상되었거나, 깨졌거나, 미완성이거나, 테스트 중이거나, 중복이거나, 오래된 맵이라도 괜찮습니다. 이 웹사이트는 그러한 문제를 알아서 처리할 수 있습니다. 또한 대개는 그런 맵 또한 스타크래프트 맵 제작 역사의 한 획들입니다. 똑같은 맵을 여러 번 업로드해도 문제가 되지 않습니다. 따라서 가진 맵을 다 업로드하고, 사이트가 필터링과 처리를 수행하도록 하십시오.",

    "Try uploading your entire StarCraft map directory, it can commonly be found at:":
      "스타크래프트 맵 디렉터리 전체를 업로드해 보세요. 보통 다음 위치에 있습니다.",

    ".scm/.scx file upload": ".scm/.scx 파일 업로드",

    "Directory (and sub directories) upload":
      "디렉터리(및 하위 디렉터리) 업로드",

    "Uploads are disabled while maintenance is being performed. Expected duration 2 hours.":
      "업로드은 연구중이기 때까지 불가하고 있습니다. 예상 시간은 2시간입니다.",

    "In Progress": "진행 중",
    Pending: "보류 중",
    Failed: "실패",
    Completed: "완료",

    Progress: "진행도",
    Size: "크기",
    "Retry All": "모두 다시 시도",
    Retry: "재시도",
    Reason: "이유",
    Link: "링크",

    // About
    "Last Updated": "마지막 업데이트",
    "Frequently Asked Questions": "자주 묻는 질문",
    "What is this website?": "이 웹사이트는 무엇인가요?",
    "Why does this website exist?": "이 웹사이트는 왜 존재하나요?",
    "What does scmscx.com mean?": "scmscx.com은(는) 무슨 뜻인가요?",
    "Why do we need another map database website?":
      "맵 데이터베이스 웹사이트가 이미 있는데 왜 새로운 것이 필요한가요?",
    "How many maps does the database have?":
      "맵 데이터베이스 웹사이트가 이미 있는데 왜 새 걸 만들었나요?",
    "Can I contribute maps to the database?":
      "데이터베이스에 맵을 제공할 수 있나요?",
    "How is this website made? What is the technology behind it?":
      "이 웹사이트는 어떻게 만들어졌나요? 그 뒤에 숨겨진 기술은 무엇인가요?",
    "Can I link directly to map downloads or minimap previews?":
      "맵 다운로드나 미니맵 미리보기에 직접 링크를 걸 수 있나요?",
    "I found a bug. How do I report it?":
      "바그를 찾아서 참고하사고, 참고하는 방법은 무엇인가요?",
    "How can I contact you?": "어떻게 연락할 수 있나요?",
    "Can I make a donation, how much does this site cost to run?":
      "기부할 수 있나요? 이 사이트를 운영하는 데 비용이 얼마나 드나요?",
    "Credit and Thanks": "제작진 및 감사의 말씀",

    // Home
    "Welcome to scmscx.com": "scmscx.com에 오신 것을 환영합니다",
    "The largest StarCraft: Brood War map database in the universe":
      "우주에서 가장 큰 스타크래프트: 브루드 워 맵 데이터베이스",

    "Recently Viewed Maps": "최근 조회된 맵",
    "Recently Downloaded Maps": "최근 다운로드된 맵",
    "Recently Uploaded Maps": "최근 업로드된 맵",
    "Recently Uploaded Replays": "최근에 업로드된 리플레이",
    "Most Viewed Maps": "가장 많이 본 맵",
    "Most Downloaded Maps": "가장 많이 다운로드한 맵",
    "Featured Maps": "추천 맵",

    // Map
    Badlands: "황무지",
    Space: "우주",
    Installation: "시설",
    Ashworld: "화산지",
    Jungle: "밀림",
    Desert: "사막",
    Ice: "얼음",
    Twilight: "황혼",

    Download: "다운로드",
    Minimap: "미니맵",
    "Scenario Properties": "시나리오 속성",
    Replays: "리플레이",
    "Known Filenames": "알려진 파일 이름",
    "Known Timestamps": "알려진 타임스탬프",
    Wavs: "Wav 파일",
    "Similar Maps": "유사한 맵",
    Flags: "플래그",
    Tags: "태그",
    Meta: "메타",

    Key: "열쇠",
    Value: "값",

    Filename: "파일 이름",
    Unit: "유닛",
    Name: "이름",

    "Last Modified Time": "마지막 수정 시간",

    "Is EUD map?": "EUD 맵인가요?",

    Duration: "지속",
    "Time Recorded": "기록된 시간",
    Creator: "창조자",

    Zerg: "저그",
    Terran: "테란",
    Protoss: "프로토스",
    Random: "무작위",
    Computer: "컴퓨터",
    Inactive: "비활성",
    Open: "열림",

    Version: "버전",
    Tileset: "타일셋",
    Dimensions: "크기",
    Locations: "로케이션",
    Doodads: "장식물(두대드)",
    Sprites: "스프라이트",
    Triggers: "트리거",
    "Briefing Triggers": "브리핑 트리거",

    Unfinished: "다듬지 않은",
    Outdated: "시대에 뒤쳐진",
    Broken: "고장난",
    "Black Holed": "블랙홀",
    "Spoiler Unit Names": "스플로러 유닛 이름",

    "MPQ Hash": "MPQ 해시",
    "MPQ Size": "MPQ 크기",
    "CHK Hash": "CHK 해시",
    "CHK Size": "CHK 크기",
    "Uploaded by": "업로드한 사람",
    "Uploaded On": "업로드 날짜",
    "Last Viewed": "마지막으로 본",
    "Last Downloaded": "마지막 다운로드",
    Views: "견해",
    Downloads: "다운로드",

    // Search Page
    Targets: "대상",
    Filters: "필터",
    Query: "질문",
    Results: "결과",

    Sorting: "종류",
    Relevancy: "관련도",
    "Last Modified (Oldest First)": "마지막 수정 시간(옛날 맵 우선)",
    "Last Modified (Newest First)": "마지막 수정 시간(최신 맵 우선)",
    "Time Uploaded (Oldest First)": "업로드 시간(옛날 맵 우선)",
    "Time Uploaded (Newest First)": "업로드 시간(최신 맵 우선)",

    Units: "유닛",
    Forces: "세력",
    Filenames: "파일 이름",
    Scenarios: "시나리오",
    "Scenario Descriptions": "시나리오 설명",

    "Minimum Map Width": "최소 맵 너비",
    "Maximum Map Width": "최대 맵 너비",

    "Minimum Map Height": "최소 맵 높이",
    "Maximum Map Height": "최대 맵 높이",

    "Maximum Human Players": "최대 인간 플레이어",
    "Minimum Human Players": "최소 인간 플레이어",

    "Maximum Computer Players": "최대 컴퓨터 플레이어",
    "Minimum Computer Players": "최소 컴퓨터 플레이어",

    "Last Modified After": "마지막 수정 시간",
    "Last Modified Before": "마지막 수정 시간",
    "Time Uploaded After": "업로드 시간",
    "Time Uploaded Before": "업로드 시간",

    Scenario: "시나리오",
    "Last Modified": "마지막 수정",
    "Time Uploaded": "업로드 시간",

    // Log in page
    Username: "사용자 이름",
    Password: "비밀번호",
    "Confirm Password": "비밀번호 확인",
    Register: "등록",

    // User page
    "New Username": "새 사용자 이름",
    "Confirm New Username": "새 사용자 이름 확인",
    "Current Password": "현재 비밀번호",
    "New Password": "새 비밀번호",
    "Confirm New Password": "새 비밀번호 확인",
  },
  zh: {
    // Navbar
    Home: "主頁",
    Search: "搜索",
    Recent: "近期",
    About: "關於",
    Upload: "上傳",
    "Change Username": "用戶名改動",
    "Change Password": "密碼改動",
    "Log out": "註銷",
    "Log in": "登入",

    // Search
    Query: "查詢",
    Random: "隨機",
    Targets: "目標",
    Filters: "篩選",
    Sorting: "排序",
    Results: "結果",

    Relevancy: "相關性",
    "Last Modified (Oldest First)": "最後修改時間(舊到新)",
    "Last Modified (Newest First)": "最後修改時間(新到舊)",
    "Time Uploaded (Oldest First)": "上傳時間(舊到新)",
    "Time Uploaded (Newest First)": "上傳時間(新到舊)",

    Units: "單位",
    Forces: "勢力",
    Filenames: "檔案名稱",
    Scenarios: "場景名（地圖名）",
    "Scenario Descriptions": "場景描述（地圖簡介）",

    Scenario: "場景名（地圖名）",
    "Last Modified": "最後修改時間",
    "Time Uploaded": "上傳時間",

    // Tileset: 地形類型
    // Badlands: "Badlands",
    // Space: "太空",
    // Installation: "Installation",
    // Ashworld: "Ashworld",
    // Jungle: "叢林",
    // Desert: "沙漠",
    // Ice: "冰地",
    // Twilight: "黃昏",

    "Minimum Map Width": "地圖寬度下限",
    "Maximum Map Width": "地圖寬度上限",
    "Minimum Map Height": "地圖高度下限",
    "Maximum Map Height": "地圖高度上限",
    "Minimum Human Players": "人類玩家數下限",
    "Maximum Human Players": "人類玩家數上限",
    "Minimum Computer Players": "電腦玩家數下限",
    "Maximum Computer Players": "電腦玩家數上限",

    "Last Modified After": "最後修改時間",
    "Last Modified Before": "最後修改時間",
    "Time Uploaded After": "上傳時間",
    "Time Uploaded Before": "上傳時間",

    // Home
    "Welcome to scmscx.com": "歡迎來到scmscx.com",
    "The largest StarCraft: Brood War map database in the universe":
      "宇宙最全的《星際爭霸：母巢之戰》地圖數據庫",

    "Recently Viewed Maps": "最近瀏覽的地圖",
    "Recently Downloaded Maps": "最近下載的地圖",
    "Recently Uploaded Maps": "最近上傳的地圖",
    "Recently Uploaded Replays": "最近上傳的replay",
    "Most Viewed Maps": "最多次查看的地圖",
    "Most Downloaded Maps": "最多次下載的地圖",
    "Featured Maps": "精選地圖",

    // Upload
    "If you want to upload one or more .scm/.scx files, then choose the top file picker.":
      "如果你想上傳一個或多個.scm/.scx檔案，使用第一個選項。",

    "If you want to upload entire directories and their sub directories, then choose the bottom file picker.":
      "如果你想上傳整個資料夾及其子目錄內所有的檔案，則使用第二個選項。",

    "Don't worry about corrupt, broken, unfinished, testing, duplicate, or outdated maps. The website will handle all of this and many of them are important parts of StarCraft map making history. Even uploading the exact same maps multiple times is no concern. So, upload everything you have and let the site do the filtering and processing.":
      "你可儘管上傳已損壞、未完成、測試用、重複或已過時的地圖，即使多次上傳完全相同的地圖也沒有問題。這些地圖可能都是你在整個地圖製作週期中的重要產物，本網站完全可以處理這些檔案。因此，請上傳你的所有內容，然後網站會自動進行過濾和處理。",

    "Try uploading your entire StarCraft map directory, it can commonly be found at:":
      "你可嘗試上傳整個《星海爭霸》的地圖資料夾，它通常位於:",

    ".scm/.scx file upload": ".scm/.scx 地圖檔案上傳",

    "Directory (and sub directories) upload": "資料夾(及其子目錄)上傳",

    "Uploads are disabled while maintenance is being performed. Expected duration 2 hours.":
      "維護中時上傳功能將被禁用。預計持續時間2小時。",

    "In Progress": "進行中",
    Pending: "等待中",
    Failed: "失敗",
    Completed: "已完成",

    Filename: "檔案名",
    Progress: "進度",
    Size: "大小",
    "Retry All": "全部重試",
    Retry: "重試",
    Reason: "原因",
    Link: "連結",

    // About
    "Last Updated": "最近更新",
    "Frequently Asked Questions": "常見問題",
    "What is this website?": "這是什麼網站？",
    "Why does this website exist?": "為什麼建立這個網站？",
    "What does scmscx.com mean?": "scmscx.com 是什麼意思？",
    "Why do we need another map database website?":
      "為什麼我們需要另一個地圖數據庫網站？",
    "How many maps does the database have?": "這個數據庫有多少地圖？",
    "Can I contribute maps to the database?": "我可以提交地圖給數據庫嗎？",
    "How is this website made? What is the technology behind it?":
      "這個網站是如何建造的？ 它的背後技術是什麼？",

    "Can I link directly to map downloads or minimap previews?":
      "我可以直接連結到地圖下載或小地圖預覽嗎？",

    "I found a bug. How do I report it?": "如何報告我發現的bug？",

    "How can I contact you?": "如何聯繫你？",

    "Can I make a donation, how much does this site cost to run?":
      "我可以捐贈嗎？這個網站的運營成本是多少？",

    "Credit and Thanks": "致謝",

    // User
    "New Username": "新用戶名稱",
    "Confirm New Username": "確認新用戶名稱",
    "Current Password": "目前密碼",

    "New Password": "新密碼",
    "Confirm New Password": "確認新密碼",

    // Map
    Download: "下載",
    Minimap: "小地圖",

    "Scenario Properties": "地圖屬性",
    Version: "版本",
    Tileset: "地形圖塊集",
    Dimensions: "尺寸",
    Locations: "位置",
    Doodads: "裝飾（小玩意）",
    Sprites: "精靈圖",
    Triggers: "觸發器",
    "Briefing Triggers": "簡報觸發器",

    Open: "空地",
    Computer: "電腦",
    Neutral: "中立",

    Replays: "回放",
    Duration: "時長",
    "Time Recorded": "記錄時間",
    Creator: "創建人",

    "Known Filenames": "已知檔案名稱",

    "Known Timestamps": "已知時間戳",
    "Last Modified Time": "最後修改時間",

    "Is EUD map?": "是否為 EUD 地圖?",
    "Get Death EUDs": "Get Death EUD數量",
    "Set Death EPDs": "Set Death EPD數量",

    Unit: "單位",
    Name: "名稱",

    Wavs: "Wav檔案",

    "Similar Maps": "相似地圖",
    Flags: "標誌",
    Unfinished: "未完成",
    Outdated: "過時",
    Broken: "破壞",
    "Black Holed": "已被黑洞吞噬",
    "Spoiler Unit Names": "懷疑單位名稱",

    Tags: "標籤",
    Key: "鍵",
    Value: "值",

    Meta: "地圖元數據",

    "MPQ Hash": "MPQ哈希",
    "MPQ Size": "MPQ大小",
    "CHK Hash": "CHK哈希",
    "CHK Size": "CHK大小",
    "Uploaded by": "上傳者",
    "Uploaded On": "上傳時間",
    "Last Viewed": "最後瀏覽時間",
    "Last Downloaded": "最後下載時間",
    Views: "瀏覽次數",
    Downloads: "下載次數",

    // Log in page
    Username: "使用者名稱",
    Password: "密碼",
    "Confirm Password": "確認密碼",
    Register: "註冊",
  },
};

const i18n_internal = (langcode: string, str: string) => {
  if (langcode === "en") {
    return str;
  } else if (langcode in langmap) {
    const langmap2 = (langmap as LangMap)[langcode] as LangMap2;

    if (str in langmap2) {
      return ((langmap as LangMap)[langcode] as LangMap2)[str];
    } else {
      return str;
    }
  } else {
    return str;
  }
};

const I18nSpan = (props: any) => {
  const [lang, _] = useLang();

  return <span>{i18n_internal(lang(), props.text)}</span>;
};

export { I18nSpan, i18n_internal };
