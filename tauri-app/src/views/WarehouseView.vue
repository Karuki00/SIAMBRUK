<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { RoomEnvironment } from "three/addons/environments/RoomEnvironment.js";
import {
  onBeforeUnmount,
  onMounted,
  reactive,
  ref,
  watch,
} from "vue";
import { ArrowLeftRight, Boxes, NotebookPen, RotateCcw } from "lucide-vue-next";
import { RouterLink } from "vue-router";
import WarehouseVariantList from "../components/WarehouseVariantList.vue";

interface Variant {
  id: number;
  product_name: string;
  name: string;
  sku: string;
  current_stock: number;
  minimum_stock: number;
  purchase_price: number;
  selling_price: number;
}
interface Summary {
  total_stock: number;
  sales_today: number;
  low_stock_count: number;
}
interface Category {
  id: number;
  name: string;
}
interface Product {
  id: number;
  name: string;
  category_id: number | null;
}
interface Lane {
  variant: Variant;
  materials: THREE.Material[];
  look: THREE.Vector3;
  normal: THREE.Vector3;
  units: THREE.Object3D[];
  tag: THREE.Mesh;
  template: THREE.Object3D;
  slotKey: string;
}
interface ItemDrag {
  id: number;
  kind: "shelf" | "bin";
  ghost: THREE.Group;
  tag: THREE.Mesh;
  tagMaterial: THREE.MeshBasicMaterial;
  zone: THREE.Object3D | null;
}
interface Box2 {
  minX: number;
  maxX: number;
  minZ: number;
  maxZ: number;
}
interface Placement {
  x: number;
  z: number;
  rotY: number;
}
interface LaneSpec {
  slots: [number, number, number][];
  tag: [number, number, number];
  tagWidth: number;
  look: [number, number, number];
}
interface Flight {
  fromPos: THREE.Vector3;
  toPos: THREE.Vector3;
  fromYaw: number;
  toYaw: number;
  fromPitch: number;
  toPitch: number;
  t: number;
  duration: number;
}
type Vec3 = [number, number, number];
type DeskTab = "category" | "product" | "variant";

/* ------------------------------------------------------------------ */
/* Layout constants. Numbers come from measuring supermarket-operations.glb */
/*   gondola: 1.26 wide x 2.08 tall x 0.55 deep, faces +z                */
/*   shelf surfaces at y .25/.70/1.15/1.60, products sit at z = +0.05    */
/*   produce bin: tray steps rise toward -z, customer side is +z         */
/* ------------------------------------------------------------------ */
const S = 1.2; // fixture scale
const ROOM = { halfW: 8, halfD: 6, height: 3.6 } as const;
/* Counter sits in the front-right corner, shopkeeper just behind it. */
const COUNTER = { x: ROOM.halfW - 2.1, z: ROOM.halfD - 1.75 } as const;
/* Area the player can walk in (inside the walls). */
const WALK = {
  minX: -ROOM.halfW + 0.5,
  maxX: ROOM.halfW - 0.5,
  minZ: -ROOM.halfD + 0.5,
  maxZ: ROOM.halfD - 0.35,
} as const;
const GONDOLA_PITCH = 1.3 * S;
const GONDOLA_BACK_Z = -0.28;
const LEVEL_Y = [0.25, 0.7, 1.15, 1.6] as const;
const TAG_Y = [0.239, 0.689, 1.139, 1.589] as const;
const SLOT_X = [-0.4, -0.2, 0, 0.2, 0.4] as const;
const ROW_Z = [0.06, -0.1] as const;
const BIN_SLOT_X = [-0.3, -0.1, 0.1, 0.3] as const;
const BIN_ROW_Y = [0.67, 0.82, 0.97] as const;
const BIN_ROW_Z = [0, -0.28, -0.56] as const;
const MIN_GONDOLAS = 8;
const MAX_BINS = 4;
const GONDOLA_TYPES = [
  "milk-carton",
  "cereal-box",
  "jam-jar",
  "water-bottle",
  "coffee-pouch",
  "soda-can",
  "flour-bag",
  "wide-food-tin",
] as const;
const PRODUCE_TYPES = ["apple", "pear", "orange"] as const;

/* Order matters: the first match wins, so specific patterns go first.
   Generic words such as "minuman" or "makanan" are intentionally left out
   because they used to turn unrelated items into cans. */
const GONDOLA_KEYWORDS: [string, RegExp][] = [
  ["milk-carton", /susu|milk|yogh?urt|krim|keju|cream/],
  [
    "cereal-box",
    /seblak|sereal|cereal|oat|granola|biskuit|biscuit|keripik|kerupuk|snack|wafer|roti|cracker|\bmie\b|\bmi\b|mi instan|indomie|bihun|soun|makaroni|pasta|spageti|kue|permen|cokelat batang|puding|bumbu instan/,
  ],
  ["jam-jar", /selai|\bjam\b|madu|sambal|saus|mayones|kecap|acar|sirup kental/],
  ["water-bottle", /\bair\b|mineral|aqua|botol|water|jus|juice|sirup|minyak/],
  [
    "coffee-pouch",
    /kopi|coffee|\bteh\b|\btea\b|kakao|coklat bubuk|bubuk|rempah|bumbu|merica|lada|ketumbar/,
  ],
  ["soda-can", /soda|cola|kaleng minuman|sprite|fanta|energi|minuman bersoda/],
  ["flour-bag", /tepung|gula|beras|garam|flour|sugar|rice|kacang|kedelai/],
  ["wide-food-tin", /sarden|tuna|kornet|\btin\b|kalengan|makanan kaleng/],
];

/* Safe fallback pool: dry-goods shapes only, never cans or bottles. */
const FALLBACK_POOL = ["cereal-box", "coffee-pouch", "flour-bag"] as const;

const PRODUCE_RE =
  /apel|apple|\bpir\b|pear|jeruk|orange|mangga|pisang|anggur|buah|fruit|melon|semangka|stroberi|strawberi/;
const FALLBACK_DIMS: Record<string, Vec3> = {
  "milk-carton": [0.094, 0.235, 0.094],
  "cereal-box": [0.16, 0.25, 0.066],
  "jam-jar": [0.1, 0.105, 0.1],
  "water-bottle": [0.07, 0.23, 0.07],
  "coffee-pouch": [0.1, 0.17, 0.054],
  "soda-can": [0.066, 0.13, 0.066],
  "flour-bag": [0.124, 0.21, 0.086],
  "wide-food-tin": [0.114, 0.075, 0.114],
  apple: [0.08, 0.098, 0.076],
  pear: [0.09, 0.135, 0.086],
  orange: [0.086, 0.086, 0.082],
};
const FALLBACK_COLORS: Record<string, number> = {
  "milk-carton": 0xf2efe4,
  "cereal-box": 0xd8903f,
  "jam-jar": 0x8b2f3c,
  "water-bottle": 0x7fb7c9,
  "coffee-pouch": 0x6b4a33,
  "soda-can": 0xc9574f,
  "flour-bag": 0xefe6d2,
  "wide-food-tin": 0x6e9188,
  apple: 0xc9574f,
  pear: 0xa8b84a,
  orange: 0xe58a2b,
};

/* Shopkeeper stands just behind the counter, facing the customer side (-z).
   A group faces +z when rotation.y = 0, so Math.PI turns it to face -z. */
const NPC_POSITION: Vec3 = [COUNTER.x + 0.6, 0, COUNTER.z + 0.9];
const NPC_ROT_Y = Math.PI;

/* Stationery desk: back-left corner, chair side faces into the room. */
const DESK_POSITION: Vec3 = [-ROOM.halfW + 0.5, 0, -ROOM.halfD + 1.2];
const DESK_INTERACT_RADIUS = 3.5;

/* Dragging products between racks */
const DRAG_THRESHOLD = 6; // px of movement before a press becomes a drag
/* v2: the shop is bigger and has more racks, so old saved slots are invalid. */
const LAYOUT_STORAGE_KEY = "siambruk-shop-layout-v2";

/* Tauri command names used by the stationery desk.
   Rename these to match the commands registered in your Rust backend. */
const CMD = {
  listCategories: "list_categories",
  createCategory: "create_category",
  listProducts: "list_products",
  createProduct: "create_product",
  createVariant: "create_variant",
} as const;
const DESK_TABS: { id: DeskTab; label: string }[] = [
  { id: "category", label: "Kategori" },
  { id: "product", label: "Produk" },
  { id: "variant", label: "Varian & stok awal" },
];

/* ------------------------------------------------------------------ */
/* Vue state                                                            */
/* ------------------------------------------------------------------ */
const host = ref<HTMLDivElement | null>(null);
const variants = ref<Variant[]>([]);
const selected = ref<Variant | null>(null);
const summary = ref<Summary | null>(null);
const quantity = ref(1);
const action = ref<"restock" | "sell">("restock");
const error = ref("");
const message = ref("");
const hovered = ref("");
const immersive = ref(false);
const dialogueOpen = ref(false);
const overflow = ref(0);

/* stationery desk modal */
const deskOpen = ref(false);
const deskTab = ref<DeskTab>("category");
const deskBusy = ref(false);
const deskError = ref("");
const deskMessage = ref("");
const categories = ref<Category[]>([]);
const products = ref<Product[]>([]);
const categoryForm = reactive({ name: "" });
const productForm = reactive({
  name: "",
  category_id: null as number | null,
});
const variantForm = reactive({
  product_id: null as number | null,
  name: "",
  sku: "",
  purchase_price: 0,
  selling_price: 0,
  minimum_stock: 5,
  initial_stock: 0,
});

/* ------------------------------------------------------------------ */
/* Three.js state                                                       */
/* ------------------------------------------------------------------ */
let scene: THREE.Scene;
let camera: THREE.PerspectiveCamera;
let renderer: THREE.WebGLRenderer;
let shopGroup: THREE.Group;
let envTexture: THREE.Texture | undefined;
let M: ReturnType<typeof makeMaterials>;
let blobGeometry: THREE.PlaneGeometry;
let blobMaterial: THREE.MeshBasicMaterial;
let tagGeometry: THREE.PlaneGeometry;
let headerGeometry: THREE.PlaneGeometry;
let gltfScene: THREE.Object3D | undefined;
let protosReady = false;
let maxAniso = 4;
let animationFrame = 0;
let resizeFrame = 0;
let resizeObserver: ResizeObserver | undefined;
let sceneOverlay: HTMLDivElement | undefined;
let syncOverlay: (() => void) | undefined;
let dragging = false;
let lastPointer = { x: 0, y: 0 };
let pendingPointer: PointerEvent | null = null;
let hoverId: number | null = null;
let skipFocus = false;
let flight: Flight | null = null;
let pointerActive = false;
let pendingDrag: number | null = null;
let itemDrag: ItemDrag | null = null;
let dragPointer = { x: 0, y: 0 };
let dropZones: THREE.Mesh[] = [];
let dropHighlight: THREE.Mesh | undefined;
let zoneGeometry: THREE.BoxGeometry;
let zoneMaterial: THREE.MeshBasicMaterial;
const dragTarget = new THREE.Vector3();
const slotOwner = new Map<string, number>();
const layout = new Map<number, string>();
let interactive: THREE.Object3D[] = [];
let interactionTargets: THREE.Object3D[] = [];
let popUnits: THREE.Object3D[] = [];
let colliders: Box2[] = [];
let staticColliders: Box2[] = [];
let npc: THREE.Group | undefined;
let npcHead: THREE.Object3D | undefined;
let npcArm: THREE.Object3D | undefined;
let npcBubble: THREE.Sprite | undefined;
let deskGroup: THREE.Group | undefined;
let deskBubble: THREE.Sprite | undefined;
let clockHour: THREE.Object3D | undefined;
let clockMinute: THREE.Object3D | undefined;
const lanes = new Map<number, Lane>();
const prototypes = new Map<string, THREE.Object3D>();
const disposables: { dispose(): void }[] = [];
let shopDisposables: { dispose(): void }[] = [];
const keys = new Set<string>();
const movementVelocity = new THREE.Vector3();
const raycaster = new THREE.Raycaster();
const pointer = new THREE.Vector2();
const clock = new THREE.Clock();
const forwardVec = new THREE.Vector3();
const rightVec = new THREE.Vector3();
const moveVec = new THREE.Vector3();
const assetLoader = new GLTFLoader();

/* ------------------------------------------------------------------ */
/* Small helpers                                                        */
/* ------------------------------------------------------------------ */
const today = () => new Date().toISOString().slice(0, 10);
const money = (value: number) =>
  new Intl.NumberFormat("id-ID", {
    style: "currency",
    currency: "IDR",
    maximumFractionDigits: 0,
  }).format(value);
function at<T>(list: readonly T[], index: number): T {
  return list[index] as T;
}
function hash(text: string) {
  let value = 0;
  for (const character of text)
    value = (value * 31 + character.charCodeAt(0)) >>> 0;
  return value;
}
function track<T extends { dispose(): void }>(item: T): T {
  disposables.push(item);
  return item;
}
function trackShop<T extends { dispose(): void }>(item: T): T {
  shopDisposables.push(item);
  return item;
}
function variantText(v: Variant) {
  return `${v.product_name} ${v.name} ${v.sku}`.toLowerCase();
}
function isProduce(v: Variant) {
  return PRODUCE_RE.test(variantText(v));
}
function matchGondola(text: string): string | undefined {
  for (const [type, pattern] of GONDOLA_KEYWORDS)
    if (pattern.test(text)) return type;
  return undefined;
}
/* Same product_name => same packaging for every variant.
   Order: product name -> full variant text -> deterministic safe fallback. */
function gondolaType(v: Variant): string {
  const productName = v.product_name.trim().toLowerCase();
  return (
    matchGondola(productName) ??
    matchGondola(variantText(v)) ??
    at(FALLBACK_POOL, hash(productName) % FALLBACK_POOL.length)
  );
}
function produceType(v: Variant): string {
  const productName = v.product_name.trim().toLowerCase();
  const text = variantText(v);
  if (/apel|apple/.test(productName) || /apel|apple/.test(text)) return "apple";
  if (/\bpir\b|pear/.test(productName) || /\bpir\b|pear/.test(text))
    return "pear";
  if (/jeruk|orange/.test(productName) || /jeruk|orange/.test(text))
    return "orange";
  return at(PRODUCE_TYPES, hash(productName) % PRODUCE_TYPES.length);
}
function byName(a: Variant, b: Variant) {
  return `${a.product_name} ${a.name}`.localeCompare(
    `${b.product_name} ${b.name}`,
  );
}
function roundRectPath(
  c: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  r: number,
) {
  c.beginPath();
  if (typeof c.roundRect === "function") c.roundRect(x, y, w, h, r);
  else c.rect(x, y, w, h);
}
function canvasTexture(
  width: number,
  height: number,
  draw: (c: CanvasRenderingContext2D, w: number, h: number) => void,
  repeat?: [number, number],
) {
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext("2d")!;
  draw(context, width, height);
  const texture = new THREE.CanvasTexture(canvas);
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.anisotropy = maxAniso;
  if (repeat) {
    texture.wrapS = THREE.RepeatWrapping;
    texture.wrapT = THREE.RepeatWrapping;
    texture.repeat.set(repeat[0], repeat[1]);
  }
  return texture;
}
function std(
  color: number,
  extra: THREE.MeshStandardMaterialParameters = {},
) {
  return track(
    new THREE.MeshStandardMaterial({
      color,
      roughness: 0.82,
      metalness: 0,
      ...extra,
    }),
  );
}
function addBox(
  parent: THREE.Object3D,
  size: Vec3,
  position: Vec3,
  material: THREE.Material,
  options: { cast?: boolean; receive?: boolean; name?: string } = {},
) {
  const mesh = new THREE.Mesh(
    track(new THREE.BoxGeometry(size[0], size[1], size[2])),
    material,
  );
  mesh.position.set(position[0], position[1], position[2]);
  mesh.castShadow = options.cast ?? true;
  mesh.receiveShadow = options.receive ?? true;
  if (options.name) mesh.name = options.name;
  parent.add(mesh);
  return mesh;
}
function addSpan(
  parent: THREE.Object3D,
  min: Vec3,
  max: Vec3,
  material: THREE.Material,
  options: { cast?: boolean; receive?: boolean; name?: string } = {},
) {
  return addBox(
    parent,
    [max[0] - min[0], max[1] - min[1], max[2] - min[2]],
    [(min[0] + max[0]) / 2, (min[1] + max[1]) / 2, (min[2] + max[2]) / 2],
    material,
    options,
  );
}
function addPlane(
  parent: THREE.Object3D,
  width: number,
  height: number,
  texture: THREE.Texture,
  position: Vec3,
  rotY = 0,
  double = false,
) {
  const mesh = new THREE.Mesh(
    track(new THREE.PlaneGeometry(width, height)),
    track(
      new THREE.MeshBasicMaterial({
        map: texture,
        toneMapped: false,
        side: double ? THREE.DoubleSide : THREE.FrontSide,
      }),
    ),
  );
  mesh.position.set(position[0], position[1], position[2]);
  mesh.rotation.y = rotY;
  parent.add(mesh);
  return mesh;
}
function enableShadows(root: THREE.Object3D) {
  root.traverse((child) => {
    if (child instanceof THREE.Mesh) {
      child.castShadow = true;
      child.receiveShadow = true;
    }
  });
}
/* Horizontal (x/z) distance from the camera; ignores eye height. */
function flatDistance(position: THREE.Vector3) {
  return Math.hypot(
    camera.position.x - position.x,
    camera.position.z - position.z,
  );
}

/* ------------------------------------------------------------------ */
/* Materials and procedural textures                                    */
/* ------------------------------------------------------------------ */
function makeMaterials() {
  const floorTexture = track(
    canvasTexture(
      512,
      512,
      (c, w, h) => {
        const tile = w / 2;
        for (let i = 0; i < 2; i += 1)
          for (let j = 0; j < 2; j += 1) {
            c.fillStyle = (i + j) % 2 === 0 ? "#dad0bb" : "#c3d2c5";
            c.fillRect(i * tile, j * tile, tile, tile);
          }
        for (let k = 0; k < 2200; k += 1) {
          c.fillStyle =
            k % 2 === 0 ? "rgba(255,255,245,0.05)" : "rgba(60,50,40,0.04)";
          c.fillRect(
            Math.random() * w,
            Math.random() * h,
            2 + Math.random() * 4,
            2 + Math.random() * 4,
          );
        }
        c.strokeStyle = "#a99d86";
        c.lineWidth = 5;
        for (let i = 0; i <= 2; i += 1) {
          c.beginPath();
          c.moveTo(i * tile, 0);
          c.lineTo(i * tile, h);
          c.moveTo(0, i * tile);
          c.lineTo(w, i * tile);
          c.stroke();
        }
      },
      [(ROOM.halfW * 2) / 1.2, (ROOM.halfD * 2) / 1.2],
    ),
  );
  const plasterTexture = track(
    canvasTexture(
      256,
      256,
      (c, w, h) => {
        c.fillStyle = "#f0e7d3";
        c.fillRect(0, 0, w, h);
        for (let k = 0; k < 1400; k += 1) {
          c.fillStyle =
            k % 2 === 0 ? "rgba(255,255,255,0.12)" : "rgba(150,130,100,0.07)";
          c.fillRect(Math.random() * w, Math.random() * h, 3, 3);
        }
      },
      [4, 2],
    ),
  );
  const woodTexture = track(
    canvasTexture(
      256,
      256,
      (c, w, h) => {
        c.fillStyle = "#a06f47";
        c.fillRect(0, 0, w, h);
        for (let k = 0; k < 70; k += 1) {
          c.strokeStyle =
            k % 3 === 0 ? "rgba(70,40,20,0.22)" : "rgba(255,220,170,0.12)";
          c.lineWidth = 1 + Math.random() * 2;
          const y = Math.random() * h;
          c.beginPath();
          c.moveTo(0, y);
          c.bezierCurveTo(w * 0.3, y + 4, w * 0.6, y - 4, w, y + 2);
          c.stroke();
        }
      },
      [2, 1],
    ),
  );
  return {
    floor: std(0xffffff, { map: floorTexture, roughness: 0.42 }),
    plaster: std(0xffffff, { map: plasterTexture, roughness: 0.95 }),
    wainscot: std(0x4c7d72, { roughness: 0.7 }),
    ceiling: std(0xf4eee0, { roughness: 0.95 }),
    beam: std(0x9b7653),
    trim: std(0x6d5340),
    teal: std(0x315e55),
    tealLight: std(0x6e9188),
    gold: std(0xd8a85f, { metalness: 0.35, roughness: 0.4 }),
    wood: std(0xffffff, { map: woodTexture }),
    dark: std(0x2b3b38),
    frame: std(0x2f4a44, { roughness: 0.5 }),
    glass: std(0xbfe3e0, {
      transparent: true,
      opacity: 0.16,
      roughness: 0.05,
      depthWrite: false,
      side: THREE.DoubleSide,
    }),
    lamp: std(0xfff1cf, {
      emissive: 0xffe3a8,
      emissiveIntensity: 2.4,
      toneMapped: false,
    }),
    skin: std(0xd9a77c, { roughness: 0.7 }),
    shirt: std(0xf1e9d6),
    cardboard: std(0xc59a62, { roughness: 0.9 }),
    tape: std(0xdcc08a, { roughness: 0.9 }),
    red: std(0xc9574f, { roughness: 0.6 }),
    green: std(0x5f896d, { roughness: 0.85 }),
    pot: std(0x8b6247),
    ground: std(0xb7bfae, { roughness: 1 }),
    black: std(0x1d2423, { roughness: 0.6 }),
    screen: std(0x1f4a44, {
      emissive: 0x3fbf9f,
      emissiveIntensity: 0.9,
      toneMapped: false,
    }),
  };
}
function signTexture(
  width: number,
  height: number,
  title: string,
  subtitle: string,
  bg: string,
  fg: string,
  titleSize: number,
) {
  return track(
    canvasTexture(width, height, (c, w, h) => {
      c.fillStyle = bg;
      roundRectPath(c, 6, 6, w - 12, h - 12, Math.min(w, h) * 0.12);
      c.fill();
      c.lineWidth = Math.max(4, h * 0.035);
      c.strokeStyle = "#d8a85f";
      c.stroke();
      c.fillStyle = fg;
      c.textAlign = "center";
      c.font = `800 ${titleSize}px 'Trebuchet MS', Arial, sans-serif`;
      c.fillText(title, w / 2, subtitle ? h * 0.52 : h * 0.64, w - 60);
      if (subtitle) {
        c.fillStyle = "#d5e6dd";
        c.font = `500 ${Math.round(titleSize * 0.42)}px Arial, sans-serif`;
        c.fillText(subtitle, w / 2, h * 0.82, w - 60);
      }
    }),
  );
}
function bubbleSprite(letter: string) {
  const texture = track(
    canvasTexture(128, 128, (c, w, h) => {
      c.fillStyle = "#d8a85f";
      c.beginPath();
      c.arc(w / 2, h / 2, 56, 0, Math.PI * 2);
      c.fill();
      c.lineWidth = 6;
      c.strokeStyle = "#fff3d6";
      c.stroke();
      c.fillStyle = "#2b3b38";
      c.font = "800 70px Arial, sans-serif";
      c.textAlign = "center";
      c.fillText(letter, w / 2, h / 2 + 24);
    }),
  );
  const sprite = new THREE.Sprite(
    track(new THREE.SpriteMaterial({ map: texture, transparent: true })),
  );
  sprite.scale.set(0.34, 0.34, 1);
  sprite.visible = false;
  return sprite;
}

/* ------------------------------------------------------------------ */
/* Room, props, lighting                                                */
/* ------------------------------------------------------------------ */
function wallBlock(parent: THREE.Object3D, min: Vec3, max: Vec3) {
  const split = 1.05;
  if (min[1] < split)
    addSpan(
      parent,
      min,
      [max[0], Math.min(max[1], split), max[2]],
      M.wainscot,
      { cast: false },
    );
  if (max[1] > split)
    addSpan(
      parent,
      [min[0], Math.max(min[1], split), min[2]],
      max,
      M.plaster,
      { cast: false },
    );
}
function addPlant(parent: THREE.Object3D, x: number, z: number, scale = 1) {
  const group = new THREE.Group();
  group.position.set(x, 0, z);
  group.scale.setScalar(scale);
  addBox(group, [0.55, 0.45, 0.55], [0, 0.225, 0], M.pot);
  const leafGeometry = track(new THREE.SphereGeometry(0.34, 12, 9));
  for (let i = 0; i < 5; i += 1) {
    const leaf = new THREE.Mesh(leafGeometry, M.green);
    const angle = (i / 5) * Math.PI * 2;
    leaf.position.set(
      Math.cos(angle) * 0.18,
      0.78 + (i % 2) * 0.22,
      Math.sin(angle) * 0.18,
    );
    leaf.scale.set(0.8, 1.25, 0.8);
    leaf.castShadow = true;
    group.add(leaf);
  }
  parent.add(group);
}
function addBasketStack(parent: THREE.Object3D, x: number, z: number) {
  const group = new THREE.Group();
  group.position.set(x, 0, z);
  addBox(group, [0.6, 0.9, 0.45], [0, 0.45, 0], M.frame);
  const handle = track(new THREE.TorusGeometry(0.15, 0.012, 6, 14, Math.PI));
  for (let i = 0; i < 3; i += 1) {
    const y = 0.98 + i * 0.045;
    addBox(group, [0.5, 0.2, 0.34], [0, y + 0.1, 0], M.red);
    addBox(group, [0.44, 0.012, 0.28], [0, y + 0.2, 0], M.black, {
      cast: false,
    });
  }
  const arc = new THREE.Mesh(handle, M.dark);
  arc.position.set(0, 1.27, 0);
  group.add(arc);
  parent.add(group);
}
function addStorageCorner(parent: THREE.Object3D) {
  const group = new THREE.Group();
  group.position.set(ROOM.halfW - 0.8, 0, -ROOM.halfD + 0.7);
  addBox(group, [1.4, 0.14, 1.1], [0, 0.07, 0], M.wood);
  const stack: [Vec3, Vec3][] = [
    [[-0.32, 0.14, -0.15], [0.55, 0.42, 0.5]],
    [[0.32, 0.14, -0.15], [0.55, 0.42, 0.5]],
    [[0, 0.56, -0.15], [0.6, 0.4, 0.5]],
    [[-0.1, 0.96, -0.12], [0.5, 0.36, 0.46]],
  ];
  for (const [offset, size] of stack) {
    addBox(
      group,
      size,
      [offset[0], offset[1] + size[1] / 2, offset[2]],
      M.cardboard,
    );
    addBox(
      group,
      [0.07, 0.01, size[2] + 0.01],
      [offset[0], offset[1] + size[1] + 0.005, offset[2]],
      M.tape,
      { cast: false },
    );
  }
  parent.add(group);
  staticColliders.push({
    minX: ROOM.halfW - 1.55,
    maxX: ROOM.halfW,
    minZ: -ROOM.halfD,
    maxZ: -ROOM.halfD + 1.3,
  });
}
function buildClock(parent: THREE.Object3D) {
  const face = track(
    canvasTexture(256, 256, (c, w, h) => {
      c.fillStyle = "#f6efdc";
      c.beginPath();
      c.arc(w / 2, h / 2, w / 2 - 8, 0, Math.PI * 2);
      c.fill();
      c.strokeStyle = "#315e55";
      c.lineWidth = 14;
      c.stroke();
      c.strokeStyle = "#2b3b38";
      for (let i = 0; i < 12; i += 1) {
        const angle = (i / 12) * Math.PI * 2;
        c.lineWidth = i % 3 === 0 ? 8 : 4;
        c.beginPath();
        c.moveTo(w / 2 + Math.sin(angle) * 92, h / 2 - Math.cos(angle) * 92);
        c.lineTo(w / 2 + Math.sin(angle) * 108, h / 2 - Math.cos(angle) * 108);
        c.stroke();
      }
    }),
  );
  const group = new THREE.Group();
  group.position.set(ROOM.halfW - 0.04, 2.45, 0.4);
  group.rotation.y = -Math.PI / 2;
  const disk = new THREE.Mesh(
    track(new THREE.CircleGeometry(0.38, 32)),
    track(new THREE.MeshBasicMaterial({ map: face, toneMapped: false })),
  );
  group.add(disk);
  const hour = new THREE.Group();
  addBox(hour, [0.025, 0.2, 0.01], [0, 0.1, 0.01], M.dark, { cast: false });
  const minute = new THREE.Group();
  addBox(minute, [0.018, 0.3, 0.01], [0, 0.15, 0.02], M.dark, { cast: false });
  group.add(hour, minute);
  clockHour = hour;
  clockMinute = minute;
  parent.add(group);
}
function buildCounter(parent: THREE.Object3D) {
  const counter = new THREE.Group();
  counter.position.set(COUNTER.x, 0, COUNTER.z);
  addBox(counter, [3.0, 1.0, 0.8], [0, 0.5, 0], M.teal);
  addBox(counter, [2.8, 0.7, 0.03], [0, 0.5, 0.41], M.tealLight);
  addBox(counter, [3.2, 0.07, 1.0], [0, 1.035, 0], M.wood);
  addBox(counter, [0.5, 0.2, 0.4], [0.9, 1.17, -0.05], M.dark);
  const screen = addBox(
    counter,
    [0.38, 0.24, 0.03],
    [0.9, 1.38, -0.2],
    M.screen,
    { cast: false },
  );
  screen.rotation.x = -0.35;
  addBox(counter, [0.16, 0.05, 0.1], [0.5, 1.1, 0.2], M.black);
  const jar = new THREE.Mesh(
    track(new THREE.CylinderGeometry(0.12, 0.12, 0.26, 16)),
    M.glass,
  );
  jar.position.set(-0.95, 1.2, 0.1);
  counter.add(jar);
  const candyGeometry = track(new THREE.SphereGeometry(0.035, 8, 6));
  for (let i = 0; i < 10; i += 1) {
    const candy = new THREE.Mesh(candyGeometry, i % 2 === 0 ? M.red : M.gold);
    candy.position.set(
      -0.95 + Math.cos(i) * 0.06,
      1.1 + (i % 3) * 0.06,
      0.1 + Math.sin(i) * 0.06,
    );
    counter.add(candy);
  }
  const bell = new THREE.Mesh(
    track(
      new THREE.SphereGeometry(0.07, 12, 8, 0, Math.PI * 2, 0, Math.PI / 2),
    ),
    M.gold,
  );
  bell.position.set(-0.3, 1.07, 0.25);
  counter.add(bell);

  /* The sign is built from two back-to-back planes, each FrontSide only.
     A single double-sided plane shows mirrored text when seen from behind;
     two planes facing opposite ways read correctly from both directions. */
  const sign = signTexture(512, 128, "KASIR", "", "#315e55", "#fff3d6", 70);
  addPlane(counter, 1.2, 0.3, sign, [0, 2.7, 0.005], 0);
  addPlane(counter, 1.2, 0.3, sign, [0, 2.7, -0.005], Math.PI);
  addBox(counter, [0.02, 1.0, 0.02], [-0.55, 3.2, 0], M.dark, { cast: false });
  addBox(counter, [0.02, 1.0, 0.02], [0.55, 3.2, 0], M.dark, { cast: false });
  parent.add(counter);
  addBlob(parent, COUNTER.x, COUNTER.z, 3.6, 1.4, 0);
  staticColliders.push({
    minX: COUNTER.x - 1.6,
    maxX: COUNTER.x + 1.7,
    minZ: COUNTER.z - 1.35,
    maxZ: COUNTER.z + 0.55,
  });
}
function buildShopkeeper(parent: THREE.Object3D) {
  const group = new THREE.Group();
  group.position.set(NPC_POSITION[0], NPC_POSITION[1], NPC_POSITION[2]);
  group.rotation.y = NPC_ROT_Y;
  group.name = "Penjaga toko";
  group.userData.interaction = "shopkeeper";
  addBox(group, [0.22, 0.8, 0.24], [-0.14, 0.4, 0], M.black);
  addBox(group, [0.22, 0.8, 0.24], [0.14, 0.4, 0], M.black);
  addBox(group, [0.62, 0.72, 0.34], [0, 1.16, 0], M.shirt);
  addBox(group, [0.5, 0.64, 0.04], [0, 1.1, 0.19], M.teal);
  const tag = signTexture(256, 64, "ALFA MABRUK", "", "#d8a85f", "#2b3b38", 34);
  addPlane(group, 0.34, 0.085, tag, [0, 1.3, 0.215]);
  addBox(group, [0.14, 0.6, 0.14], [-0.38, 1.12, 0], M.shirt);
  const armPivot = new THREE.Group();
  armPivot.position.set(0.38, 1.44, 0);
  addBox(armPivot, [0.14, 0.6, 0.14], [0, -0.28, 0], M.shirt);
  const hand = new THREE.Mesh(
    track(new THREE.SphereGeometry(0.08, 10, 8)),
    M.skin,
  );
  hand.position.set(0, -0.62, 0);
  armPivot.add(hand);
  group.add(armPivot);
  const handLeft = hand.clone();
  handLeft.position.set(-0.38, 0.5, 0);
  group.add(handLeft);
  const head = new THREE.Group();
  head.position.set(0, 1.74, 0);
  const skull = new THREE.Mesh(
    track(new THREE.SphereGeometry(0.21, 18, 14)),
    M.skin,
  );
  head.add(skull);
  const peci = new THREE.Mesh(
    track(new THREE.CylinderGeometry(0.17, 0.2, 0.16, 18)),
    M.black,
  );
  peci.position.set(0, 0.2, 0);
  head.add(peci);
  const eyeGeometry = track(new THREE.SphereGeometry(0.026, 8, 6));
  for (const side of [-1, 1]) {
    const eye = new THREE.Mesh(eyeGeometry, M.black);
    eye.position.set(side * 0.075, 0.03, 0.185);
    head.add(eye);
  }
  const smile = new THREE.Mesh(
    track(new THREE.TorusGeometry(0.06, 0.008, 6, 12, Math.PI)),
    M.black,
  );
  smile.position.set(0, -0.06, 0.19);
  smile.rotation.z = Math.PI;
  head.add(smile);
  group.add(head);
  const bubble = bubbleSprite("E");
  bubble.position.set(0, 2.3, 0);
  group.add(bubble);
  enableShadows(group);
  parent.add(group);
  npc = group;
  npcHead = head;
  npcArm = armPivot;
  npcBubble = bubble;
  interactionTargets.push(group);
}
/* Stationery desk with a chair, in the back-left corner. Clicking it, or
   pressing E nearby, opens the catalog modal (category / product / variant). */
function buildStationeryDesk(parent: THREE.Object3D) {
  const group = new THREE.Group();
  group.position.set(DESK_POSITION[0], DESK_POSITION[1], DESK_POSITION[2]);
  group.rotation.y = Math.PI / 2; /* local +z (chair side) faces +x, the room */
  group.name = "Meja alat tulis";
  group.userData.interaction = "desk";

  /* desk top and legs */
  addBox(group, [1.4, 0.05, 0.7], [0, 0.76, 0], M.wood);
  for (const x of [-0.62, 0.62])
    for (const z of [-0.29, 0.29])
      addBox(group, [0.05, 0.74, 0.05], [x, 0.37, z], M.dark);
  addBox(group, [1.3, 0.2, 0.03], [0, 0.62, -0.3], M.teal);

  /* chair */
  addBox(group, [0.46, 0.05, 0.46], [0, 0.45, 0.85], M.tealLight);
  addBox(group, [0.46, 0.5, 0.05], [0, 0.72, 1.08], M.tealLight);
  for (const x of [-0.19, 0.19])
    for (const z of [0.66, 1.04])
      addBox(group, [0.04, 0.43, 0.04], [x, 0.215, z], M.dark);

  /* stationery on the desk: notebook, folder, pen cup, lamp */
  addBox(group, [0.3, 0.025, 0.22], [-0.35, 0.8, 0.05], M.red);
  addBox(group, [0.24, 0.06, 0.32], [0.05, 0.82, -0.08], M.shirt);
  const cup = new THREE.Mesh(
    track(new THREE.CylinderGeometry(0.05, 0.045, 0.12, 12)),
    M.dark,
  );
  cup.position.set(0.5, 0.85, -0.2);
  group.add(cup);
  [M.gold, M.red, M.green].forEach((pen, i) =>
    addBox(group, [0.012, 0.12, 0.012], [0.485 + i * 0.015, 0.97, -0.2], pen, {
      cast: false,
    }),
  );
  addBox(group, [0.14, 0.03, 0.14], [-0.55, 0.8, -0.22], M.dark);
  addBox(group, [0.02, 0.36, 0.02], [-0.55, 0.98, -0.22], M.dark, {
    cast: false,
  });
  addBox(group, [0.16, 0.04, 0.12], [-0.55, 1.17, -0.22], M.lamp, {
    cast: false,
  });

  const bubble = bubbleSprite("E");
  bubble.position.set(0, 1.7, 0.3);
  group.add(bubble);

  parent.add(group);
  addPlane(
    parent,
    1.1,
    0.62,
    signTexture(
      512,
      288,
      "ALAT TULIS",
      "kategori · produk",
      "#2f574e",
      "#fff3d6",
      84,
    ),
    [-ROOM.halfW + 0.03, 1.9, DESK_POSITION[2]],
    Math.PI / 2,
  );
  addBlob(parent, DESK_POSITION[0] + 0.5, DESK_POSITION[2], 1.8, 2.0, 0);
  staticColliders.push({
    minX: -ROOM.halfW,
    maxX: -ROOM.halfW + 1.65,
    minZ: -ROOM.halfD,
    maxZ: -ROOM.halfD + 1.95,
  });
  deskGroup = group;
  deskBubble = bubble;
  interactionTargets.push(group);
}
function addBlob(
  parent: THREE.Object3D,
  x: number,
  z: number,
  width: number,
  depth: number,
  rotY: number,
) {
  const blob = new THREE.Mesh(blobGeometry, blobMaterial);
  blob.rotation.x = -Math.PI / 2;
  blob.rotation.z = rotY;
  blob.position.set(x, 0.004, z);
  blob.scale.set(width, depth, 1);
  parent.add(blob);
}
function buildRoom() {
  scene = new THREE.Scene();
  const sky = new THREE.Color(0xcfe3e0);
  scene.background = sky;
  scene.fog = new THREE.Fog(sky, 24, 80);
  camera = new THREE.PerspectiveCamera(55, 1, 0.05, 100);
  camera.rotation.order = "YXZ";
  renderer = new THREE.WebGLRenderer({
    antialias: true,
    powerPreference: "high-performance",
  });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.75));
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFShadowMap;
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.0;
  host.value?.appendChild(renderer.domElement);
  maxAniso = renderer.capabilities.getMaxAnisotropy();

  const pmrem = new THREE.PMREMGenerator(renderer);
  envTexture = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
  scene.environment = envTexture;
  Object.assign(scene, { environmentIntensity: 0.45 });
  pmrem.dispose();

  M = makeMaterials();
  blobGeometry = track(new THREE.PlaneGeometry(1, 1));
  blobMaterial = track(
    new THREE.MeshBasicMaterial({
      map: track(
        canvasTexture(128, 128, (c, w, h) => {
          const g = c.createRadialGradient(w / 2, h / 2, 6, w / 2, h / 2, w / 2);
          g.addColorStop(0, "rgba(25,30,28,0.55)");
          g.addColorStop(0.6, "rgba(25,30,28,0.22)");
          g.addColorStop(1, "rgba(25,30,28,0)");
          c.fillStyle = g;
          c.fillRect(0, 0, w, h);
        }),
      ),
      transparent: true,
      depthWrite: false,
      toneMapped: false,
    }),
  );
  tagGeometry = track(new THREE.PlaneGeometry(1, 0.105));
  headerGeometry = track(new THREE.PlaneGeometry(0.9, 0.22));
  zoneGeometry = track(new THREE.BoxGeometry(1, 1, 1));
  zoneMaterial = track(new THREE.MeshBasicMaterial({ visible: false }));
  dropHighlight = new THREE.Mesh(
    zoneGeometry,
    track(
      new THREE.MeshBasicMaterial({
        color: 0x6fd0b0,
        transparent: true,
        opacity: 0.28,
        depthWrite: false,
        toneMapped: false,
      }),
    ),
  );
  dropHighlight.visible = false;
  dropHighlight.renderOrder = 5;
  scene.add(dropHighlight);

  /* lights */
  scene.add(new THREE.HemisphereLight(0xfff4e0, 0x8a7f6a, 0.6));
  const sun = new THREE.DirectionalLight(0xfff2d6, 2.2);
  sun.position.set(-12, 11, 7);
  sun.castShadow = true;
  sun.shadow.mapSize.set(2048, 2048);
  sun.shadow.camera.left = -11;
  sun.shadow.camera.right = 11;
  sun.shadow.camera.top = 11;
  sun.shadow.camera.bottom = -11;
  sun.shadow.camera.near = 1;
  sun.shadow.camera.far = 40;
  sun.shadow.bias = -0.0004;
  sun.shadow.normalBias = 0.03;
  scene.add(sun, sun.target);

  const roomGroup = new THREE.Group();
  scene.add(roomGroup);
  const H = ROOM.height;
  const W = ROOM.halfW;
  const D = ROOM.halfD;

  /* ground + floor */
  const ground = new THREE.Mesh(
    track(new THREE.PlaneGeometry(160, 160)),
    M.ground,
  );
  ground.rotation.x = -Math.PI / 2;
  ground.position.y = -0.03;
  ground.receiveShadow = true;
  roomGroup.add(ground);
  const floor = new THREE.Mesh(
    track(new THREE.PlaneGeometry(W * 2, D * 2)),
    M.floor,
  );
  floor.rotation.x = -Math.PI / 2;
  floor.receiveShadow = true;
  roomGroup.add(floor);

  /* walls */
  wallBlock(roomGroup, [-W - 0.2, 0, -D - 0.2], [W + 0.2, H, -D]);
  wallBlock(roomGroup, [W, 0, -D], [W + 0.2, H, D + 0.2]);
  wallBlock(roomGroup, [-W - 0.2, 0, -D], [-W, H, -1.2]);
  wallBlock(roomGroup, [-W - 0.2, 0, -1.2], [-W, 0.95, 3.4]);
  wallBlock(roomGroup, [-W - 0.2, 2.7, -1.2], [-W, H, 3.4]);
  wallBlock(roomGroup, [-W - 0.2, 0, 3.4], [-W, H, D + 0.2]);
  wallBlock(roomGroup, [-W - 0.2, 0, D], [-0.95, 0.95, D + 0.2]);
  wallBlock(roomGroup, [0.95, 0, D], [W + 0.2, 0.95, D + 0.2]);
  wallBlock(roomGroup, [-W - 0.2, 2.7, D], [W + 0.2, H, D + 0.2]);
  /* trims */
  addSpan(roomGroup, [-W, 0, -D], [W, 0.12, -D + 0.04], M.trim, { cast: false });
  addSpan(roomGroup, [W - 0.04, 0, -D], [W, 0.12, D], M.trim, { cast: false });
  addSpan(roomGroup, [-W, 1.03, -D], [W, 1.09, -D + 0.03], M.gold, {
    cast: false,
  });
  addSpan(roomGroup, [W - 0.03, 1.03, -D], [W, 1.09, D], M.gold, {
    cast: false,
  });
  /* windows: left wall and front wall */
  const leftGlass = new THREE.Mesh(
    track(new THREE.PlaneGeometry(4.6, 1.75)),
    M.glass,
  );
  leftGlass.rotation.y = Math.PI / 2;
  leftGlass.position.set(-W - 0.1, 1.825, 1.1);
  roomGroup.add(leftGlass);
  for (const z of [-1.2, 0.33, 1.87, 3.4])
    addBox(roomGroup, [0.1, 1.75, 0.07], [-W - 0.1, 1.825, z], M.frame, {
      cast: false,
    });
  addBox(roomGroup, [0.1, 0.05, 4.6], [-W - 0.1, 1.8, 1.1], M.frame, {
    cast: false,
  });
  const frontGlass = new THREE.Mesh(
    track(new THREE.PlaneGeometry(W * 2, 1.75)),
    M.glass,
  );
  frontGlass.position.set(0, 1.825, D + 0.1);
  roomGroup.add(frontGlass);
  for (const x of [-7.2, -5.6, -4, -2.4, -0.95, 0.95, 2.4, 4, 5.6, 7.2])
    addBox(roomGroup, [0.08, 1.75, 0.1], [x, 1.825, D + 0.1], M.frame, {
      cast: false,
    });
  addBox(roomGroup, [W * 2, 0.05, 0.1], [0, 1.8, D + 0.1], M.frame, {
    cast: false,
  });
  /* door */
  addBox(roomGroup, [0.1, 2.7, 0.14], [-0.95, 1.35, D + 0.1], M.dark, {
    cast: false,
  });
  addBox(roomGroup, [0.1, 2.7, 0.14], [0.95, 1.35, D + 0.1], M.dark, {
    cast: false,
  });
  addBox(roomGroup, [1.9, 0.1, 0.14], [0, 2.65, D + 0.1], M.dark, {
    cast: false,
  });
  addBox(roomGroup, [0.04, 0.5, 0.05], [0.7, 1.05, D + 0.05], M.gold, {
    cast: false,
  });
  addBox(roomGroup, [0.04, 0.5, 0.05], [-0.7, 1.05, D + 0.05], M.gold, {
    cast: false,
  });
  addPlane(
    roomGroup,
    0.7,
    0.34,
    signTexture(256, 128, "BUKA", "silakan masuk", "#315e55", "#fff3d6", 58),
    [0, 1.6, D + 0.12],
    Math.PI,
    true,
  );

  /* ceiling, beams, lamps */
  addSpan(
    roomGroup,
    [-W - 0.2, H, -D - 0.2],
    [W + 0.2, H + 0.1, D + 0.2],
    M.ceiling,
    { cast: false },
  );
  for (const z of [-4.4, -1.4, 1.6, 4.6])
    addSpan(roomGroup, [-W, H - 0.14, z - 0.1], [W, H, z + 0.1], M.beam, {
      cast: false,
    });
  for (const x of [-5, 0, 5])
    for (const z of [-3.5, 0.5, 4])
      addBox(roomGroup, [1.4, 0.05, 0.5], [x, H - 0.03, z], M.lamp, {
        cast: false,
      });
  /* a handful of real lights is enough; the rest is baked into the lamps */
  for (const [x, z] of [
    [-4, -3],
    [4, -3],
    [-4, 2.5],
    [4, 2.5],
  ] as const) {
    const light = new THREE.PointLight(0xffe2b0, 14, 12, 2);
    light.position.set(x, H - 0.4, z);
    roomGroup.add(light);
  }
  const warm = new THREE.PointLight(0xffe2b0, 10, 10, 2);
  warm.position.set(0, H - 0.4, -1);
  roomGroup.add(warm);

  /* back-wall sign, rug, posters */
  addPlane(
    roomGroup,
    6,
    1.1,
    signTexture(
      1536,
      282,
      "TOKO ALFA MABRUK",
      "Stok nyata · harga jelas",
      "#2f574e",
      "#fff3d6",
      128,
    ),
    [0, 3.0, -D + 0.03],
  );
  const rugTexture = track(
    canvasTexture(512, 256, (c, w, h) => {
      c.fillStyle = "#315e55";
      c.fillRect(0, 0, w, h);
      c.strokeStyle = "#d8a85f";
      c.lineWidth = 8;
      c.strokeRect(16, 16, w - 32, h - 32);
      c.fillStyle = "#fff3d6";
      c.font = "800 44px 'Trebuchet MS', Arial, sans-serif";
      c.textAlign = "center";
      c.fillText("SELAMAT DATANG", w / 2, h / 2 + 15);
    }),
  );
  const rug = new THREE.Mesh(
    track(new THREE.PlaneGeometry(3.4, 1.7)),
    track(new THREE.MeshBasicMaterial({ map: rugTexture, toneMapped: false })),
  );
  rug.rotation.x = -Math.PI / 2;
  rug.position.set(0, 0.008, D - 1);
  roomGroup.add(rug);
  const posters: [string, string, string][] = [
    ["PROMO", "pekan ini", "#c9574f"],
    ["KOPI", "hangat setiap pagi", "#6b4a33"],
    ["SEGAR", "buah pilihan", "#5f896d"],
  ];
  posters.forEach(([title, sub, bg], i) => {
    addPlane(
      roomGroup,
      0.9,
      1.25,
      signTexture(360, 500, title, sub, bg, "#fff3d6", 78),
      [W - 0.03, 2.1, -3.2 + i * 1.35],
      -Math.PI / 2,
    );
  });
  buildClock(roomGroup);

  /* props */
  addPlant(roomGroup, -1.9, D - 0.45, 1.1);
  addPlant(roomGroup, 1.9, D - 0.45, 1.1);
  addPlant(roomGroup, -W + 0.5, -D + 2.6, 1.1);
  addBasketStack(roomGroup, 2.0, D - 1);
  addStorageCorner(roomGroup);
  buildCounter(roomGroup);
  buildShopkeeper(roomGroup);
  buildStationeryDesk(roomGroup);

  shopGroup = new THREE.Group();
  scene.add(shopGroup);
  resetCamera();
}

/* ------------------------------------------------------------------ */
/* GLB prototypes and fallbacks                                         */
/* ------------------------------------------------------------------ */
function collectPrototypes(root: THREE.Object3D) {
  root.traverse((object) => {
    const type = object.name.match(/^\d+-([a-z-]+)$/)?.[1];
    if (!type || prototypes.has(type)) return;
    const copy = object.clone(true);
    copy.position.set(0, 0, 0);
    copy.rotation.set(0, 0, 0);
    copy.scale.set(1, 1, 1);
    copy.visible = true;
    prototypes.set(type, copy);
  });
}
function fallbackGondola() {
  const group = new THREE.Group();
  const white = std(0xefe6d2);
  addBox(group, [1.26, 2.0, 0.06], [0, 1.0, -0.25], M.dark);
  for (const x of [-0.6, 0.6])
    addBox(group, [0.06, 2.0, 0.56], [x, 1.0, -0.01], M.dark);
  for (const y of [0.2, 0.65, 1.1, 1.55, 2.0]) {
    addBox(group, [1.2, 0.05, 0.52], [0, y + 0.025, -0.01], white);
    addBox(group, [1.2, 0.08, 0.03], [0, y + 0.039, 0.25], M.teal);
  }
  return group;
}
function fallbackBin() {
  const group = new THREE.Group();
  for (const x of [-0.5, 0.5])
    for (const z of [-0.3, 0.3])
      addBox(group, [0.05, 0.6, 0.05], [x, 0.3, z], M.dark);
  for (let k = 0; k < 4; k += 1) {
    addBox(group, [1.2, 0.05, 0.28], [0, 0.645 + 0.15 * k, 0.01 - 0.28 * k], M.wood);
    addBox(group, [1.2, 0.15, 0.03], [0, 0.57 + 0.15 * k, 0.15 - 0.28 * k], M.wood);
  }
  return group;
}
function fallbackProduct(type: string) {
  const group = new THREE.Group();
  const dims = FALLBACK_DIMS[type] ?? [0.1, 0.15, 0.1];
  addBox(
    group,
    dims,
    [0, dims[1] / 2, 0],
    std(FALLBACK_COLORS[type] ?? 0xc9574f, { roughness: 0.6 }),
  );
  return group;
}
function ensurePrototypes() {
  if (!prototypes.has("single-sided-gondola"))
    prototypes.set("single-sided-gondola", fallbackGondola());
  if (!prototypes.has("produce-tilt-bin"))
    prototypes.set("produce-tilt-bin", fallbackBin());
  for (const type of [...GONDOLA_TYPES, ...PRODUCE_TYPES])
    if (!prototypes.has(type)) prototypes.set(type, fallbackProduct(type));
}
function loadFixtureAsset() {
  assetLoader.load(
    "/assets/3d/supermarket-operations.glb",
    (gltf) => {
      gltfScene = gltf.scene;
      collectPrototypes(gltf.scene);
      ensurePrototypes();
      protosReady = true;
      rebuildShop();
    },
    undefined,
    () => {
      error.value =
        "Model toko lokal tidak dapat dimuat; mode cadangan digunakan.";
      ensurePrototypes();
      protosReady = true;
      rebuildShop();
    },
  );
}
function proto(type: string): THREE.Object3D {
  return prototypes.get(type) ?? new THREE.Group();
}

/* ------------------------------------------------------------------ */
/* Shop fixtures and data-driven products                               */
/* ------------------------------------------------------------------ */
/* 8 racks along the back wall + two islands of 3+3 back-to-back racks
   (20 racks in total, 80 shelf levels). */
function gondolaPlacements(): Placement[] {
  const list: Placement[] = [];
  const wallZ = -ROOM.halfD + 0.05 - GONDOLA_BACK_Z * S;
  for (let i = 0; i < 8; i += 1)
    list.push({ x: (i - 3.5) * GONDOLA_PITCH, z: wallZ, rotY: 0 });
  const islandZ = -1.6;
  const half = -GONDOLA_BACK_Z * S + 0.02;
  for (const cx of [-3.2, 3.2]) {
    for (let i = 0; i < 3; i += 1)
      list.push({
        x: cx + (i - 1) * GONDOLA_PITCH,
        z: islandZ + half,
        rotY: 0,
      });
    for (let i = 0; i < 3; i += 1)
      list.push({
        x: cx + (i - 1) * GONDOLA_PITCH,
        z: islandZ - half,
        rotY: Math.PI,
      });
  }
  return list;
}
function binPlacements(): Placement[] {
  const list: Placement[] = [];
  const x = -ROOM.halfW + 0.05 + 0.71 * S;
  for (let i = 0; i < MAX_BINS; i += 1)
    list.push({
      x,
      z: ROOM.halfD - 1.3 - i * GONDOLA_PITCH,
      rotY: Math.PI / 2,
    });
  return list;
}
function ownMaterials(root: THREE.Object3D, store: THREE.Material[]) {
  const cache = new Map<THREE.Material, THREE.Material>();
  const own = (material: THREE.Material) => {
    let copy = cache.get(material);
    if (!copy) {
      copy = material.clone();
      cache.set(material, copy);
      store.push(copy);
    }
    return copy;
  };
  root.traverse((child) => {
    if (child instanceof THREE.Mesh) {
      child.material = Array.isArray(child.material)
        ? child.material.map(own)
        : own(child.material as THREE.Material);
      child.castShadow = true;
      child.receiveShadow = true;
    }
  });
}
function tagTexture(v: Variant) {
  const empty = v.current_stock <= 0;
  const low = !empty && v.current_stock <= v.minimum_stock;
  return trackShop(
    canvasTexture(768, 80, (c, w, h) => {
      c.fillStyle = empty ? "#5b2a26" : low ? "#a8352b" : "#315e55";
      roundRectPath(c, 3, 3, w - 6, h - 6, 12);
      c.fill();
      c.lineWidth = 5;
      c.strokeStyle = "#d8a85f";
      c.stroke();
      c.fillStyle = "#fff3d6";
      c.textBaseline = "middle";
      c.textAlign = "left";
      c.font = "800 34px 'Trebuchet MS', Arial, sans-serif";
      c.fillText(`${v.product_name} ${v.name}`, 22, h / 2, 440);
      c.textAlign = "right";
      c.font = "700 30px Arial, sans-serif";
      c.fillText(
        empty
          ? "HABIS"
          : `${money(v.selling_price)} · stok ${v.current_stock}`,
        w - 22,
        h / 2,
        290,
      );
    }),
  );
}
function addLane(
  fixture: THREE.Group,
  rotY: number,
  v: Variant,
  type: string,
  spec: LaneSpec,
  slotKey: string,
) {
  const materials: THREE.Material[] = [];
  const template = proto(type).clone(true);
  ownMaterials(template, materials);
  materials.forEach((material) => trackShop(material));
  const units: THREE.Object3D[] = [];
  const count = Math.min(
    Math.max(Math.floor(v.current_stock), 0),
    spec.slots.length,
  );
  for (let i = 0; i < count; i += 1) {
    const slot = at(spec.slots, i);
    const unit = template.clone(true);
    unit.position.set(slot[0], slot[1], slot[2]);
    unit.rotation.y = ((hash(`${v.id}-${i}`) % 100) / 100 - 0.5) * 0.3;
    unit.scale.setScalar(0.001);
    unit.userData.variantId = v.id;
    unit.userData.pop = -i * 0.03;
    fixture.add(unit);
    interactive.push(unit);
    popUnits.push(unit);
    units.push(unit);
  }
  const tagMaterial = trackShop(
    new THREE.MeshBasicMaterial({ map: tagTexture(v), toneMapped: false }),
  );
  const tag = new THREE.Mesh(tagGeometry, tagMaterial);
  tag.position.set(spec.tag[0], spec.tag[1], spec.tag[2]);
  tag.scale.set(spec.tagWidth, 1, 1);
  tag.userData.variantId = v.id;
  fixture.add(tag);
  interactive.push(tag);
  const look = fixture.localToWorld(
    new THREE.Vector3(spec.look[0], spec.look[1], spec.look[2]),
  );
  lanes.set(v.id, {
    variant: v,
    materials,
    look,
    normal: new THREE.Vector3(Math.sin(rotY), 0, Math.cos(rotY)),
    units,
    tag,
    template,
    slotKey,
  });
  slotOwner.set(slotKey, v.id);
}
/* Invisible box over one shelf level (or produce bin). Only used to find
   where a dragged product is dropped; it is never rendered. */
function addDropZone(
  fixture: THREE.Group,
  key: string,
  center: Vec3,
  size: Vec3,
  anchor: Vec3,
) {
  const zone = new THREE.Mesh(zoneGeometry, zoneMaterial);
  zone.position.set(center[0], center[1], center[2]);
  zone.scale.set(size[0], size[1], size[2]);
  zone.userData.slotKey = key;
  zone.userData.kind = key.startsWith("b:") ? "bin" : "shelf";
  zone.userData.anchor = anchor;
  fixture.add(zone);
  dropZones.push(zone);
}
function slotLabel(key: string) {
  const [kind, first = "0", second = "0"] = key.split(":");
  if (kind === "b") return `keranjang buah ${Number(first) + 1}`;
  return `Rak ${String.fromCharCode(65 + Number(first))} tingkat ${Number(second) + 1}`;
}

/* Layout overrides: variant id -> slot key ("s:gondola:level" or "b:bin").
   Stored on this device so the arrangement survives reloads. */
function loadLayout() {
  try {
    const raw = window.localStorage.getItem(LAYOUT_STORAGE_KEY);
    if (!raw) return;
    const data = JSON.parse(raw) as Record<string, unknown>;
    layout.clear();
    for (const [id, key] of Object.entries(data))
      if (typeof key === "string" && /^(s:\d+:\d+|b:\d+)$/.test(key))
        layout.set(Number(id), key);
  } catch {
    /* storage unavailable or corrupted: fall back to the default layout */
  }
}
function saveLayout() {
  try {
    window.localStorage.setItem(
      LAYOUT_STORAGE_KEY,
      JSON.stringify(Object.fromEntries(layout)),
    );
  } catch {
    /* storage unavailable: the layout just won't persist */
  }
}
function resetLayout() {
  layout.clear();
  saveLayout();
  error.value = "";
  message.value = "Susunan rak dikembalikan ke bawaan.";
  rebuildShop();
}
/* Decide which variant sits in which slot.
   1) saved positions, 2) the default slot for that sort position,
   3) the first free slot for anything left over. */
function assignSlots(ids: number[], slotKeys: string[]) {
  const owner = new Map<string, number>();
  const keySet = new Set(slotKeys);
  const placed = new Set<number>();
  for (const id of ids) {
    const key = layout.get(id);
    if (key && keySet.has(key) && !owner.has(key)) {
      owner.set(key, id);
      placed.add(id);
    }
  }
  ids.forEach((id, index) => {
    if (placed.has(id)) return;
    const preferred = slotKeys[index];
    if (preferred && !owner.has(preferred)) {
      owner.set(preferred, id);
      placed.add(id);
    }
  });
  for (const id of ids) {
    if (placed.has(id)) continue;
    const free = slotKeys.find((key) => !owner.has(key));
    if (!free) break;
    owner.set(free, id);
    placed.add(id);
  }
  return owner;
}
function clearShop() {
  while (shopGroup.children.length) shopGroup.remove(shopGroup.children[0]!);
  shopDisposables.forEach((item) => item.dispose());
  shopDisposables = [];
  lanes.clear();
  slotOwner.clear();
  dropZones = [];
  interactive = [];
  popUnits = [];
  colliders = [...staticColliders];
  hoverId = null;
}
function rebuildShop() {
  if (!scene || !protosReady) return;
  endItemDrag(false);
  clearShop();
  const all = variants.value.slice();
  const produce = all.filter(isProduce).sort(byName);
  /* Sorting by package type groups identical packaging on the same shelf. */
  const dry = all
    .filter((v) => !isProduce(v))
    .map((v) => ({ v, type: gondolaType(v) }))
    .sort(
      (a, b) =>
        GONDOLA_TYPES.indexOf(a.type as (typeof GONDOLA_TYPES)[number]) -
          GONDOLA_TYPES.indexOf(b.type as (typeof GONDOLA_TYPES)[number]) ||
        byName(a.v, b.v),
    );
  const dryById = new Map(dry.map((entry) => [entry.v.id, entry] as const));
  const produceById = new Map(produce.map((v) => [v.id, v] as const));
  const placements = gondolaPlacements();
  const gondolaCount = Math.min(
    placements.length,
    Math.max(MIN_GONDOLAS, Math.ceil(dry.length / 4)),
  );
  overflow.value =
    Math.max(0, dry.length - placements.length * 4) +
    Math.max(0, produce.length - MAX_BINS);

  const fixtures: THREE.Group[] = [];
  for (let g = 0; g < gondolaCount; g += 1) {
    const p = at(placements, g);
    const fixture = new THREE.Group();
    fixture.position.set(p.x, 0, p.z);
    fixture.rotation.y = p.rotY;
    fixture.scale.setScalar(S);
    const body = proto("single-sided-gondola").clone(true);
    enableShadows(body);
    fixture.add(body);
    const blob = new THREE.Mesh(blobGeometry, blobMaterial);
    blob.rotation.x = -Math.PI / 2;
    blob.position.set(0, 0.004, 0.05);
    blob.scale.set(1.75, 1.1, 1);
    fixture.add(blob);
    const headerMaterial = trackShop(
      new THREE.MeshBasicMaterial({
        map: trackShop(
          canvasTexture(384, 94, (c, w, h) => {
            c.fillStyle = "#d8a85f";
            roundRectPath(c, 4, 4, w - 8, h - 8, 14);
            c.fill();
            c.fillStyle = "#2b3b38";
            c.textAlign = "center";
            c.font = "800 52px 'Trebuchet MS', Arial, sans-serif";
            c.fillText(`RAK ${String.fromCharCode(65 + g)}`, w / 2, h / 2 + 18);
          }),
        ),
        toneMapped: false,
      }),
    );
    const header = new THREE.Mesh(headerGeometry, headerMaterial);
    header.position.set(0, 2.22, 0.1);
    fixture.add(header);
    shopGroup.add(fixture);
    fixture.updateMatrixWorld(true);
    fixtures.push(fixture);
    colliders.push({
      minX: p.x - 0.78,
      maxX: p.x + 0.78,
      minZ: p.z - 0.36,
      maxZ: p.z + 0.36,
    });
  }

  /* shelf levels: one drop zone each, then place products by saved layout */
  const shelfKeys: string[] = [];
  for (let g = 0; g < gondolaCount; g += 1)
    for (let level = 0; level < 4; level += 1) {
      const key = `s:${g}:${level}`;
      shelfKeys.push(key);
      addDropZone(
        at(fixtures, g),
        key,
        [0, at(LEVEL_Y, level) + 0.2, 0],
        [1.2, 0.44, 0.56],
        [0, at(LEVEL_Y, level) + 0.02, 0.06],
      );
    }
  assignSlots(
    dry.map((entry) => entry.v.id),
    shelfKeys,
  ).forEach((id, key) => {
    const entry = dryById.get(id);
    if (!entry) return;
    const [, gText = "0", levelText = "0"] = key.split(":");
    const g = Number(gText);
    const level = Number(levelText);
    const p = at(placements, g);
    const slots: Vec3[] = [];
    for (let i = 0; i < 10; i += 1)
      slots.push([
        at(SLOT_X, i % 5),
        at(LEVEL_Y, level),
        at(ROW_Z, Math.floor(i / 5)),
      ]);
    addLane(
      at(fixtures, g),
      p.rotY,
      entry.v,
      entry.type,
      {
        slots,
        tag: [0, at(TAG_Y, level), 0.268],
        tagWidth: 1.0,
        look: [0, at(LEVEL_Y, level) + 0.15, 0.05],
      },
      key,
    );
  });

  /* produce bins */
  const binSpots = binPlacements();
  const binCount = Math.min(MAX_BINS, Math.max(3, produce.length));
  const binKeys = Array.from({ length: binCount }, (_, b) => `b:${b}`);
  const binOwner = assignSlots(
    produce.map((v) => v.id),
    binKeys,
  );
  for (let b = 0; b < binCount; b += 1) {
    const p = at(binSpots, b);
    const fixture = new THREE.Group();
    fixture.position.set(p.x, 0, p.z);
    fixture.rotation.y = p.rotY;
    fixture.scale.setScalar(S);
    const body = proto("produce-tilt-bin").clone(true);
    enableShadows(body);
    fixture.add(body);
    const blob = new THREE.Mesh(blobGeometry, blobMaterial);
    blob.rotation.x = -Math.PI / 2;
    blob.position.set(0, 0.004, -0.2);
    blob.scale.set(1.5, 1.3, 1);
    fixture.add(blob);
    shopGroup.add(fixture);
    fixture.updateMatrixWorld(true);
    colliders.push({
      minX: -ROOM.halfW,
      maxX: p.x + 0.325 * S + 0.05,
      minZ: p.z - 0.74,
      maxZ: p.z + 0.74,
    });
    const key = at(binKeys, b);
    addDropZone(
      fixture,
      key,
      [0, 0.82, -0.28],
      [1.2, 0.55, 0.95],
      [0, 0.75, -0.25],
    );
    const id = binOwner.get(key);
    const v = id === undefined ? undefined : produceById.get(id);
    if (!v) continue;
    const slots: Vec3[] = [];
    for (let i = 0; i < 12; i += 1)
      slots.push([
        at(BIN_SLOT_X, i % 4),
        at(BIN_ROW_Y, Math.floor(i / 4)),
        at(BIN_ROW_Z, Math.floor(i / 4)),
      ]);
    addLane(
      fixture,
      p.rotY,
      v,
      produceType(v),
      {
        slots,
        tag: [0, 0.61, 0.178],
        tagWidth: 1.1,
        look: [0, 0.8, -0.28],
      },
      key,
    );
  }
  shopGroup.updateMatrixWorld(true);
  applyHighlight();
}

/* ------------------------------------------------------------------ */
/* Highlight, selection, focus                                          */
/* ------------------------------------------------------------------ */
function setEmissive(
  materials: THREE.Material[],
  color: number,
  intensity: number,
) {
  for (const material of materials) {
    const standard = material as THREE.MeshStandardMaterial;
    if (standard.emissive) {
      standard.emissive.setHex(color);
      standard.emissiveIntensity = intensity;
    }
  }
}
function applyHighlight() {
  const selectedId = selected.value?.id ?? null;
  lanes.forEach((lane, id) => {
    if (id === selectedId) setEmissive(lane.materials, 0xd8a85f, 0.4);
    else if (id === hoverId) setEmissive(lane.materials, 0x6fd0b0, 0.3);
    else setEmissive(lane.materials, 0x000000, 0);
  });
}
function findTarget(object: THREE.Object3D | undefined) {
  let target = object;
  while (
    target &&
    target.userData.variantId === undefined &&
    !target.userData.interaction
  )
    target = target.parent ?? undefined;
  return target;
}
function selectFromObject(object: THREE.Object3D | undefined) {
  const target = findTarget(object);
  if (target?.userData.interaction === "desk") {
    openDesk();
    return;
  }
  if (target?.userData.interaction === "shopkeeper") {
    dialogueOpen.value = true;
    return;
  }
  const id = target?.userData.variantId as number | undefined;
  if (id === undefined) return;
  if (selected.value?.id !== id) skipFocus = true;
  selected.value = variants.value.find((variant) => variant.id === id) ?? null;
}
function shortestAngle(delta: number) {
  const full = Math.PI * 2;
  return ((((delta + Math.PI) % full) + full) % full) - Math.PI;
}
function focusLane(id: number) {
  const lane = lanes.get(id);
  if (!lane || !camera) return;
  const to = new THREE.Vector3(
    lane.look.x + lane.normal.x * 2.5,
    THREE.MathUtils.clamp(lane.look.y + 0.5, 1.45, 2.1),
    lane.look.z + lane.normal.z * 2.5,
  );
  to.x = THREE.MathUtils.clamp(to.x, WALK.minX, WALK.maxX);
  to.z = THREE.MathUtils.clamp(to.z, WALK.minZ, WALK.maxZ);
  if (Math.hypot(camera.position.x - to.x, camera.position.z - to.z) < 0.5)
    return;
  const dx = lane.look.x - to.x;
  const dz = lane.look.z - to.z;
  flight = {
    fromPos: camera.position.clone(),
    toPos: to,
    fromYaw: camera.rotation.y,
    toYaw: Math.atan2(-dx, -dz),
    fromPitch: camera.rotation.x,
    toPitch: THREE.MathUtils.clamp(
      Math.atan2(lane.look.y - to.y, Math.hypot(dx, dz)),
      -0.5,
      0.5,
    ),
    t: 0,
    duration: 0.9,
  };
}
watch(
  () => selected.value?.id,
  (id) => {
    applyHighlight();
    if (skipFocus) {
      skipFocus = false;
      return;
    }
    if (id !== undefined) focusLane(id);
  },
);
watch([immersive, dialogueOpen, variants, summary], () => syncOverlay?.());

/* ------------------------------------------------------------------ */
/* Overlay, pointer, keyboard                                           */
/* ------------------------------------------------------------------ */
function setImmersive(value: boolean) {
  immersive.value = value;
  document.body.classList.toggle("immersive-active", value);
  if (!value) document.exitPointerLock?.();
}
function setupSceneOverlay() {
  if (!host.value) return;
  const overlay = document.createElement("div");
  sceneOverlay = overlay;
  overlay.className = "scene-overlay";
  const toggle = document.createElement("button");
  toggle.className = "scene-immersive-toggle";
  toggle.textContent = "Masuk Mode Imersif";
  toggle.onclick = () => setImmersive(!immersive.value);
  overlay.append(toggle);
  const hud = document.createElement("div");
  hud.className = "scene-hud";
  hud.innerHTML = `<strong>SIAMBRUK</strong><span>Ruang toko · Zona penjualan</span><div class="scene-hud-stats"></div><button type="button">Keluar mode</button>`;
  hud
    .querySelector("button")!
    .addEventListener("click", () => setImmersive(false));
  overlay.append(hud);
  const dialogue = document.createElement("div");
  dialogue.className = "scene-dialogue";
  dialogue.innerHTML = `<strong>Penjaga Toko Alfa Mabruk</strong><p>Selamat datang di Toko Alfa Mabruk!</p><p>Saya bisa membantu Anda mengenal ruang inventori SIAMBRUK.</p><button type="button">Tutup dialog</button>`;
  dialogue.querySelector("button")!.addEventListener("click", () => {
    dialogueOpen.value = false;
  });
  overlay.append(dialogue);
  host.value.append(overlay);
  syncOverlay = () => {
    hud.classList.toggle("visible", immersive.value);
    toggle.textContent = immersive.value
      ? "Keluar Mode Imersif"
      : "Masuk Mode Imersif";
    dialogue.classList.toggle("visible", dialogueOpen.value);
    const stats = hud.querySelector(".scene-hud-stats");
    if (stats)
      stats.textContent = `${variants.value.length} varian · ${summary.value?.total_stock ?? "—"} unit · ${summary.value?.low_stock_count ?? "—"} stok rendah · ${summary.value?.sales_today ?? "—"} terjual hari ini`;
  };
  syncOverlay();
}
function setRay(clientX: number, clientY: number) {
  const rect = renderer.domElement.getBoundingClientRect();
  pointer.x = ((clientX - rect.left) / rect.width) * 2 - 1;
  pointer.y = -((clientY - rect.top) / rect.height) * 2 + 1;
  raycaster.setFromCamera(pointer, camera);
}
function cast(event: PointerEvent) {
  setRay(event.clientX, event.clientY);
  return raycaster.intersectObjects(
    [...interactive, ...interactionTargets],
    true,
  );
}
function onPointerMove(event: PointerEvent) {
  pendingPointer = event;
}
function processHover() {
  const event = pendingPointer;
  pendingPointer = null;
  if (!event || dragging || itemDrag) return;
  const target = findTarget(cast(event)[0]?.object);
  const id = target?.userData.variantId as number | undefined;
  const nextHover = id ?? null;
  if (nextHover !== hoverId) {
    hoverId = nextHover;
    applyHighlight();
  }
  if (id !== undefined) {
    const lane = lanes.get(id);
    hovered.value = lane
      ? `${lane.variant.product_name} ${lane.variant.name}`
      : "";
  } else {
    hovered.value = target?.userData.interaction ? target.name : "";
  }
  renderer.domElement.style.cursor = target ? "pointer" : "grab";
}
function onPointerDown(event: PointerEvent) {
  pointerActive = true;
  flight = null;
  lastPointer = { x: event.clientX, y: event.clientY };
  dragPointer = { x: event.clientX, y: event.clientY };
  const target = findTarget(cast(event)[0]?.object);
  const id = target?.userData.variantId as number | undefined;
  if (id !== undefined && lanes.has(id)) {
    /* pressed on a product: a click selects it, a drag moves it */
    pendingDrag = id;
    dragging = false;
    renderer.domElement.style.cursor = "pointer";
  } else {
    pendingDrag = null;
    dragging = true;
    renderer.domElement.style.cursor = "grabbing";
  }
}
function onPointerUp(event: PointerEvent) {
  if (!pointerActive) return;
  pointerActive = false;
  if (itemDrag) {
    endItemDrag(true);
  } else if (
    Math.hypot(event.clientX - lastPointer.x, event.clientY - lastPointer.y) <
    DRAG_THRESHOLD
  ) {
    selectFromObject(cast(event)[0]?.object);
  }
  pendingDrag = null;
  dragging = false;
  renderer.domElement.style.cursor = "grab";
}
function onPointerCancel() {
  endItemDrag(false);
  pointerActive = false;
  pendingDrag = null;
  dragging = false;
  renderer.domElement.style.cursor = "grab";
}
function onPointerDrag(event: PointerEvent) {
  dragPointer = { x: event.clientX, y: event.clientY };
  if (pendingDrag !== null && !itemDrag) {
    if (
      Math.hypot(event.clientX - lastPointer.x, event.clientY - lastPointer.y) >=
      DRAG_THRESHOLD
    )
      beginItemDrag(pendingDrag);
  }
  if (itemDrag) return;
  if (!dragging) return;
  camera.rotation.y -= (event.clientX - lastPointer.x) * 0.006;
  camera.rotation.x = THREE.MathUtils.clamp(
    camera.rotation.x - (event.clientY - lastPointer.y) * 0.004,
    -0.7,
    0.6,
  );
  lastPointer = { x: event.clientX, y: event.clientY };
}

/* ------------------------------------------------------------------ */
/* Dragging products between racks                                      */
/* ------------------------------------------------------------------ */
function beginItemDrag(id: number) {
  const lane = lanes.get(id);
  if (!lane) {
    pendingDrag = null;
    return;
  }
  /* select it so the side panel shows what is being moved */
  const variant = variants.value.find((item) => item.id === id);
  if (variant && selected.value?.id !== id) {
    skipFocus = true;
    selected.value = variant;
  }
  hoverId = null;
  hovered.value = "";

  /* ghost: a few copies of the product, with the price tag floating above */
  const ghost = new THREE.Group();
  ghost.scale.setScalar(S);
  const count = Math.max(1, Math.min(lane.units.length, 5));
  for (let i = 0; i < count; i += 1) {
    const unit = lane.template.clone(true);
    unit.position.set((i - (count - 1) / 2) * 0.2, 0, 0);
    unit.rotation.set(0, 0, 0);
    unit.scale.setScalar(1);
    ghost.add(unit);
  }
  const topY = new THREE.Box3().setFromObject(ghost).max.y / S;
  const tagMaterial = new THREE.MeshBasicMaterial({
    map: (lane.tag.material as THREE.MeshBasicMaterial).map,
    toneMapped: false,
    transparent: true,
    depthTest: false,
    depthWrite: false,
  });
  const tag = new THREE.Mesh(tagGeometry, tagMaterial);
  tag.scale.set(lane.tag.scale.x * 0.85, 0.85, 1);
  tag.position.set(0, topY + 0.13, 0);
  tag.renderOrder = 999;
  ghost.add(tag);
  scene.add(ghost);

  /* hide the originals while the ghost is in the air */
  lane.units.forEach((unit) => {
    unit.visible = false;
  });
  lane.tag.visible = false;

  itemDrag = {
    id,
    kind: lane.slotKey.startsWith("b:") ? "bin" : "shelf",
    ghost,
    tag,
    tagMaterial,
    zone: null,
  };
  updateItemDrag(0, true);
  renderer.domElement.style.cursor = "grabbing";
}
function showDropHighlight(zone: THREE.Object3D, sourceKey?: string) {
  if (!dropHighlight) return;
  const key = zone.userData.slotKey as string;
  if (key === sourceKey) {
    dropHighlight.visible = false;
    return;
  }
  zone.updateWorldMatrix(true, false);
  zone.matrixWorld.decompose(
    dropHighlight.position,
    dropHighlight.quaternion,
    dropHighlight.scale,
  );
  /* gold = the slot is taken and the two products will swap */
  (dropHighlight.material as THREE.MeshBasicMaterial).color.setHex(
    slotOwner.has(key) ? 0xd8a85f : 0x6fd0b0,
  );
  dropHighlight.visible = true;
}
function updateItemDrag(dt: number, snap = false) {
  const drag = itemDrag;
  if (!drag) return;
  setRay(dragPointer.x, dragPointer.y);
  const hit = raycaster
    .intersectObjects(dropZones, false)
    .find((item) => item.object.userData.kind === drag.kind);
  const lane = lanes.get(drag.id);
  let yaw = camera.rotation.y;
  if (hit) {
    const zone = hit.object;
    const anchor = zone.userData.anchor as Vec3;
    const fixture = zone.parent as THREE.Object3D;
    dragTarget.set(anchor[0], anchor[1], anchor[2]);
    fixture.localToWorld(dragTarget);
    yaw = fixture.rotation.y;
    drag.zone = zone;
    showDropHighlight(zone, lane?.slotKey);
  } else {
    drag.zone = null;
    dragTarget
      .copy(raycaster.ray.direction)
      .multiplyScalar(2.4)
      .add(raycaster.ray.origin);
    dragTarget.y = Math.max(dragTarget.y - 0.1, 0.5);
    if (dropHighlight) dropHighlight.visible = false;
  }
  const k = snap ? 1 : 1 - Math.exp(-dt * 16);
  drag.ghost.position.lerp(dragTarget, k);
  drag.ghost.rotation.y += shortestAngle(yaw - drag.ghost.rotation.y) * k;
  /* keep the price tag facing the camera, whatever the ghost's yaw */
  drag.tag.quaternion
    .copy(drag.ghost.quaternion)
    .invert()
    .multiply(camera.quaternion);
}
function endItemDrag(commit: boolean) {
  const drag = itemDrag;
  if (!drag) return;
  itemDrag = null;
  scene.remove(drag.ghost);
  drag.tagMaterial.dispose();
  if (dropHighlight) dropHighlight.visible = false;
  const lane = lanes.get(drag.id);
  const key = drag.zone?.userData.slotKey as string | undefined;
  if (commit && lane && key && key !== lane.slotKey) {
    moveLane(drag.id, key);
    return;
  }
  if (lane) {
    lane.units.forEach((unit) => {
      unit.visible = true;
    });
    lane.tag.visible = true;
  }
}
/* Move to an empty slot, or swap with whatever is already there. */
function moveLane(id: number, targetKey: string) {
  const lane = lanes.get(id);
  if (!lane) return;
  const sourceKey = lane.slotKey;
  const occupantId = slotOwner.get(targetKey);
  const name = `${lane.variant.product_name} ${lane.variant.name}`;
  layout.set(id, targetKey);
  let text = `${name} dipindah ke ${slotLabel(targetKey)}.`;
  if (occupantId !== undefined && occupantId !== id) {
    layout.set(occupantId, sourceKey);
    const other = lanes.get(occupantId);
    if (other)
      text = `${name} ditukar dengan ${other.variant.product_name} ${other.variant.name}.`;
  }
  saveLayout();
  error.value = "";
  message.value = text;
  rebuildShop();
}
function onWheel(event: WheelEvent) {
  flight = null;
  const yaw = camera.rotation.y;
  const amount = -event.deltaY * 0.006;
  tryMove(-Math.sin(yaw) * amount, -Math.cos(yaw) * amount);
}
function isTyping(event: KeyboardEvent) {
  const target = event.target;
  return (
    target instanceof HTMLElement &&
    (["INPUT", "SELECT", "TEXTAREA"].includes(target.tagName) ||
      target.isContentEditable)
  );
}
function onKeyDown(event: KeyboardEvent) {
  /* Escape closes the desk modal even while typing in one of its fields. */
  if (event.key === "Escape" && deskOpen.value) {
    closeDesk();
    return;
  }
  /* While the modal is open, the scene ignores the keyboard. */
  if (deskOpen.value || isTyping(event)) return;
  const key = event.key.toLowerCase();
  if (key === "escape" && itemDrag) {
    endItemDrag(false);
    return;
  }
  if (key === "escape" && immersive.value) {
    setImmersive(false);
    return;
  }
  if (key === "e") {
    if (npc && camera.position.distanceTo(npc.position) < 4.5)
      dialogueOpen.value = true;
    else if (deskGroup && flatDistance(deskGroup.position) < DESK_INTERACT_RADIUS)
      openDesk();
  }
  keys.add(key);
}
function onKeyUp(event: KeyboardEvent) {
  keys.delete(event.key.toLowerCase());
}
function clearKeys() {
  keys.clear();
  onPointerCancel();
}

/* ------------------------------------------------------------------ */
/* Camera, movement, animation loop                                     */
/* ------------------------------------------------------------------ */
function resetCamera() {
  if (!camera) return;
  flight = null;
  camera.position.set(0, 2.15, ROOM.halfD - 0.6);
  camera.rotation.set(-0.12, 0, 0);
  movementVelocity.set(0, 0, 0);
}
function blocked(x: number, z: number) {
  const r = 0.32;
  if (x < WALK.minX || x > WALK.maxX || z < WALK.minZ || z > WALK.maxZ)
    return true;
  return colliders.some(
    (b) => x > b.minX - r && x < b.maxX + r && z > b.minZ - r && z < b.maxZ + r,
  );
}
function tryMove(dx: number, dz: number) {
  const nextX = camera.position.x + dx;
  if (!blocked(nextX, camera.position.z)) camera.position.x = nextX;
  const nextZ = camera.position.z + dz;
  if (!blocked(camera.position.x, nextZ)) camera.position.z = nextZ;
}
function updateMovement(dt: number) {
  const yaw = camera.rotation.y;
  forwardVec.set(-Math.sin(yaw), 0, -Math.cos(yaw));
  rightVec.set(Math.cos(yaw), 0, -Math.sin(yaw));
  moveVec.set(0, 0, 0);
  if (keys.has("w") || keys.has("arrowup")) moveVec.add(forwardVec);
  if (keys.has("s") || keys.has("arrowdown")) moveVec.sub(forwardVec);
  if (keys.has("d") || keys.has("arrowright")) moveVec.add(rightVec);
  if (keys.has("a") || keys.has("arrowleft")) moveVec.sub(rightVec);
  if (moveVec.lengthSq() > 0) {
    flight = null;
    moveVec.normalize().multiplyScalar(3.4);
  }
  movementVelocity.lerp(moveVec, 1 - Math.exp(-dt * 9));
  if (movementVelocity.lengthSq() > 1e-6)
    tryMove(movementVelocity.x * dt, movementVelocity.z * dt);
}
function updateFlight(dt: number) {
  if (!flight) return;
  flight.t += dt / flight.duration;
  const k = Math.min(flight.t, 1);
  const e = k * k * (3 - 2 * k);
  camera.position.lerpVectors(flight.fromPos, flight.toPos, e);
  camera.rotation.y =
    flight.fromYaw + shortestAngle(flight.toYaw - flight.fromYaw) * e;
  camera.rotation.x = flight.fromPitch + (flight.toPitch - flight.fromPitch) * e;
  if (k >= 1) flight = null;
}
function updatePops(dt: number) {
  if (!popUnits.length) return;
  popUnits = popUnits.filter((unit) => {
    const pop = ((unit.userData.pop as number) ?? 0) + dt * 2.6;
    unit.userData.pop = pop;
    if (pop < 0) return true;
    const k = Math.min(pop, 1);
    const c1 = 1.70158;
    const c3 = c1 + 1;
    const eased = 1 + c3 * Math.pow(k - 1, 3) + c1 * Math.pow(k - 1, 2);
    unit.scale.setScalar(Math.max(eased, 0.001));
    return k < 1;
  });
}
function animate() {
  animationFrame = requestAnimationFrame(animate);
  const dt = Math.min(clock.getDelta(), 0.05);
  const t = clock.elapsedTime;
  updateMovement(dt);
  updateFlight(dt);
  updateItemDrag(dt);
  processHover();
  updatePops(dt);
  if (npc && npcHead && npcArm && npcBubble) {
    const near = camera.position.distanceTo(npc.position) < 4.5;
    npcBubble.visible = near;
    npcBubble.position.y = 2.3 + Math.sin(t * 3) * 0.03;
    npcHead.rotation.y = Math.sin(t * 0.7) * 0.3;
    npc.position.y = Math.sin(t * 1.6) * 0.006;
    const raise = near ? 2.5 + Math.sin(t * 7) * 0.25 : 0.05;
    npcArm.rotation.z += (raise - npcArm.rotation.z) * 0.12;
  }
  if (deskGroup && deskBubble) {
    const near =
      !deskOpen.value && flatDistance(deskGroup.position) < DESK_INTERACT_RADIUS;
    deskBubble.visible = near;
    deskBubble.position.y = 1.7 + Math.sin(t * 3) * 0.03;
  }
  if (clockHour && clockMinute) {
    const now = new Date();
    clockMinute.rotation.z =
      -((now.getMinutes() + now.getSeconds() / 60) / 60) * Math.PI * 2;
    clockHour.rotation.z =
      -(((now.getHours() % 12) + now.getMinutes() / 60) / 12) * Math.PI * 2;
  }
  const selectedId = selected.value?.id;
  if (selectedId !== undefined) {
    const lane = lanes.get(selectedId);
    if (lane)
      setEmissive(lane.materials, 0xd8a85f, 0.3 + Math.sin(t * 4) * 0.12);
  }
  renderer.render(scene, camera);
}
function resize() {
  if (!host.value || !renderer) return;
  const width = host.value.clientWidth;
  const height = host.value.clientHeight;
  if (!width || !height) return;
  renderer.setSize(width, height, false);
  camera.aspect = width / height;
  camera.updateProjectionMatrix();
}
function scheduleResize() {
  cancelAnimationFrame(resizeFrame);
  resizeFrame = requestAnimationFrame(resize);
}

/* ------------------------------------------------------------------ */
/* Data and transactions                                                */
/* ------------------------------------------------------------------ */
async function load() {
  try {
    [variants.value, summary.value] = await Promise.all([
      invoke<Variant[]>("list_variants", { productId: null }),
      invoke<Summary>("report_summary"),
    ]);
    rebuildShop();
  } catch (e) {
    error.value = String(e);
  }
}
async function transact() {
  const current = selected.value;
  if (!current || quantity.value <= 0) return;
  error.value = "";
  message.value = "";
  try {
    const input =
      action.value === "restock"
        ? {
            reference: "3D shop",
            purchased_at: today(),
            notes: "Restock dari ruang 3D",
            items: [
              {
                variant_id: current.id,
                quantity: quantity.value,
                unit_cost: current.purchase_price,
              },
            ],
          }
        : {
            reference: "3D shop",
            sold_at: today(),
            notes: "Penjualan dari ruang 3D",
            items: [
              {
                variant_id: current.id,
                quantity: quantity.value,
                unit_price: current.selling_price,
              },
            ],
          };
    await invoke(
      action.value === "restock" ? "create_purchase" : "create_sale",
      { input },
    );
    message.value =
      action.value === "restock"
        ? "Stok berhasil ditambah."
        : "Penjualan berhasil dicatat.";
    await load();
    selected.value =
      variants.value.find((variant) => variant.id === current.id) ?? null;
    applyHighlight();
  } catch (e) {
    error.value = String(e);
  }
}

/* ------------------------------------------------------------------ */
/* Stationery desk modal: categories, products, variants                */
/* ------------------------------------------------------------------ */
async function loadDeskData() {
  try {
    [categories.value, products.value] = await Promise.all([
      invoke<Category[]>(CMD.listCategories),
      invoke<Product[]>(CMD.listProducts),
    ]);
  } catch (e) {
    deskError.value = String(e);
  }
}
function openDesk() {
  clearKeys();
  document.exitPointerLock?.();
  deskError.value = "";
  deskMessage.value = "";
  deskOpen.value = true;
  void loadDeskData();
}
function closeDesk() {
  deskOpen.value = false;
}
async function runDesk(work: () => Promise<string>) {
  if (deskBusy.value) return;
  deskBusy.value = true;
  deskError.value = "";
  deskMessage.value = "";
  try {
    deskMessage.value = await work();
  } catch (e) {
    deskError.value = String(e);
  } finally {
    deskBusy.value = false;
  }
}
async function submitCategory() {
  const name = categoryForm.name.trim();
  if (!name) {
    deskError.value = "Nama kategori wajib diisi.";
    return;
  }
  await runDesk(async () => {
    await invoke(CMD.createCategory, { input: { name } });
    categoryForm.name = "";
    await loadDeskData();
    return `Kategori "${name}" dibuat.`;
  });
}
async function submitProduct() {
  const name = productForm.name.trim();
  if (!name) {
    deskError.value = "Nama produk wajib diisi.";
    return;
  }
  await runDesk(async () => {
    await invoke(CMD.createProduct, {
      input: { name, category_id: productForm.category_id },
    });
    productForm.name = "";
    await loadDeskData();
    return `Produk "${name}" dibuat.`;
  });
}
async function submitVariant() {
  const name = variantForm.name.trim();
  if (variantForm.product_id === null || !name) {
    deskError.value = "Pilih produk dan isi nama varian.";
    return;
  }
  await runDesk(async () => {
    const sku =
      variantForm.sku.trim() || `SKU-${Date.now().toString(36).toUpperCase()}`;
    const created = await invoke<{ id: number } | null>(CMD.createVariant, {
      input: {
        product_id: variantForm.product_id,
        name,
        sku,
        purchase_price: variantForm.purchase_price,
        selling_price: variantForm.selling_price,
        minimum_stock: variantForm.minimum_stock,
      },
    });
    /* opening stock goes through the normal purchase flow so history stays consistent */
    if (variantForm.initial_stock > 0 && created?.id !== undefined)
      await invoke("create_purchase", {
        input: {
          reference: "Meja alat tulis",
          purchased_at: today(),
          notes: "Stok awal dari meja alat tulis",
          items: [
            {
              variant_id: created.id,
              quantity: variantForm.initial_stock,
              unit_cost: variantForm.purchase_price,
            },
          ],
        },
      });
    variantForm.name = "";
    variantForm.sku = "";
    await load(); /* refresh racks so the new variant appears on a shelf */
    return `Varian "${name}" dibuat.`;
  });
}

/* ------------------------------------------------------------------ */
/* Lifecycle                                                            */
/* ------------------------------------------------------------------ */
onMounted(async () => {
  loadLayout();
  buildRoom();
  setupSceneOverlay();
  loadFixtureAsset();
  resize();
  resizeObserver = new ResizeObserver(scheduleResize);
  if (host.value) resizeObserver.observe(host.value);
  renderer.domElement.style.cursor = "grab";
  renderer.domElement.style.touchAction = "none";
  renderer.domElement.addEventListener("pointermove", onPointerMove);
  renderer.domElement.addEventListener("pointerdown", onPointerDown);
  renderer.domElement.addEventListener("wheel", onWheel, { passive: true });
  window.addEventListener("pointermove", onPointerDrag);
  window.addEventListener("pointerup", onPointerUp);
  window.addEventListener("pointercancel", onPointerCancel);
  window.addEventListener("keydown", onKeyDown);
  window.addEventListener("keyup", onKeyUp);
  window.addEventListener("blur", clearKeys);
  animate();
  await load();
});
onBeforeUnmount(() => {
  cancelAnimationFrame(animationFrame);
  cancelAnimationFrame(resizeFrame);
  endItemDrag(false);
  resizeObserver?.disconnect();
  sceneOverlay?.remove();
  syncOverlay = undefined;
  document.body.classList.remove("immersive-active");
  renderer?.domElement.removeEventListener("pointermove", onPointerMove);
  renderer?.domElement.removeEventListener("pointerdown", onPointerDown);
  renderer?.domElement.removeEventListener("wheel", onWheel);
  window.removeEventListener("pointermove", onPointerDrag);
  window.removeEventListener("pointerup", onPointerUp);
  window.removeEventListener("pointercancel", onPointerCancel);
  window.removeEventListener("keydown", onKeyDown);
  window.removeEventListener("keyup", onKeyUp);
  window.removeEventListener("blur", clearKeys);
  shopDisposables.forEach((item) => item.dispose());
  shopDisposables = [];
  disposables.forEach((item) => item.dispose());
  disposables.length = 0;
  gltfScene?.traverse((object) => {
    if (object instanceof THREE.Mesh) {
      object.geometry.dispose();
      const materials = Array.isArray(object.material)
        ? object.material
        : [object.material as THREE.Material];
      materials.forEach((material) => material.dispose());
    }
  });
  envTexture?.dispose();
  renderer?.domElement.remove();
  renderer?.dispose();
});
</script>

<template>
  <section class="module-page">
    <div class="section-intro">
      <div>
        <p class="eyebrow">Eksplorasi interaktif</p>
        <h2>Ruang 3D inventori</h2>
        <p class="intro-copy">
          Jelajahi toko, pilih paket produk, lalu lakukan operasi dengan data
          inventori yang sama.
        </p>
      </div>
      <div class="warehouse-actions">
        <button class="secondary-button" type="button" @click="resetCamera">
          <RotateCcw :size="15" /> Reset kamera
        </button>
        <button class="secondary-button" type="button" @click="resetLayout">
          Reset susunan rak
        </button>
        <button class="secondary-button" type="button" @click="openDesk">
          <NotebookPen :size="15" /> Meja alat tulis
        </button>
        <RouterLink class="secondary-button" to="/inventory">
          <ArrowLeftRight :size="15" /> Buka tabel inventori
        </RouterLink>
      </div>
    </div>
    <p v-if="error" class="form-error">{{ error }}</p>
    <p v-if="message" class="form-success">{{ message }}</p>
    <div class="warehouse-layout">
      <div ref="host" class="three-stage" aria-label="Ruang 3D inventori">
        <span v-if="hovered" class="scene-hint"
          >Klik untuk memilih {{ hovered }}</span
        ><span class="scene-controls"
          >WASD bergerak · tarik mouse melihat · roda zoom · E bicara / buka
          meja · seret produk untuk pindah rak</span
        >
      </div>
      <aside class="panel warehouse-control">
        <div class="panel-heading">
          <div>
            <p class="eyebrow">Objek terpilih</p>
            <h3>{{ selected ? selected.name : "Pilih paket" }}</h3>
          </div>
          <Boxes :size="18" class="panel-icon" />
        </div>
        <div v-if="selected" class="selected-info">
          <span>{{ selected.product_name }} · {{ selected.sku }}</span
          ><strong>Stok {{ selected.current_stock }}</strong
          ><span
            >Beli {{ money(selected.purchase_price) }} · Jual
            {{ money(selected.selling_price) }}</span
          ><label
            >Aksi<select v-model="action">
              <option value="restock">Restock</option>
              <option value="sell">Catat penjualan</option>
            </select></label
          ><label
            >Jumlah<input
              v-model.number="quantity"
              type="number"
              min="1" /></label
          ><button class="primary-button" type="button" @click="transact">
            Konfirmasi operasi
          </button>
        </div>
        <div v-else class="table-state">
          <span>Klik paket di rak untuk melihat stok dan aksi.</span>
        </div>
      </aside>
    </div>
    <p v-if="overflow > 0" class="motion-note">
      Rak penuh: {{ overflow }} varian tidak ditampilkan di ruang 3D. Semua
      varian tetap tersedia di daftar di bawah.
    </p>
    <WarehouseVariantList
      :variants="variants"
      :selected-id="selected?.id ?? null"
      @select="selected = $event"
    />
    <p class="motion-note">
      Visual mengikuti database setelah operasi berhasil. Gunakan daftar
      alternatif untuk akses keyboard dan pengelolaan standar.
    </p>

    <Teleport to="body">
      <div v-if="deskOpen" class="desk-backdrop" @click.self="closeDesk">
        <div
          class="desk-modal"
          role="dialog"
          aria-modal="true"
          aria-label="Meja alat tulis"
        >
          <header class="desk-head">
            <div>
              <p class="eyebrow">Meja alat tulis</p>
              <h3>Kelola katalog</h3>
            </div>
            <button class="secondary-button" type="button" @click="closeDesk">
              Tutup
            </button>
          </header>
          <nav class="desk-tabs" aria-label="Jenis data">
            <button
              v-for="tab in DESK_TABS"
              :key="tab.id"
              type="button"
              :class="{ active: deskTab === tab.id }"
              @click="deskTab = tab.id"
            >
              {{ tab.label }}
            </button>
          </nav>
          <p v-if="deskError" class="form-error">{{ deskError }}</p>
          <p v-if="deskMessage" class="form-success">{{ deskMessage }}</p>

          <form
            v-if="deskTab === 'category'"
            class="desk-form"
            @submit.prevent="submitCategory"
          >
            <label
              >Nama kategori
              <input
                v-model="categoryForm.name"
                type="text"
                placeholder="mis. Makanan ringan"
            /></label>
            <button class="primary-button" type="submit" :disabled="deskBusy">
              Buat kategori
            </button>
            <div v-if="categories.length" class="desk-chips">
              <span v-for="c in categories" :key="c.id">{{ c.name }}</span>
            </div>
            <p v-else class="desk-empty">
              Belum ada kategori. Buat yang pertama di atas.
            </p>
          </form>

          <form
            v-else-if="deskTab === 'product'"
            class="desk-form"
            @submit.prevent="submitProduct"
          >
            <label
              >Nama produk
              <input
                v-model="productForm.name"
                type="text"
                placeholder="mis. Seblak"
            /></label>
            <label
              >Kategori
              <select v-model="productForm.category_id">
                <option :value="null">Tanpa kategori</option>
                <option v-for="c in categories" :key="c.id" :value="c.id">
                  {{ c.name }}
                </option>
              </select>
            </label>
            <button class="primary-button" type="submit" :disabled="deskBusy">
              Buat produk
            </button>
          </form>

          <form v-else class="desk-form" @submit.prevent="submitVariant">
            <label
              >Produk
              <select v-model="variantForm.product_id">
                <option :value="null" disabled>Pilih produk</option>
                <option v-for="p in products" :key="p.id" :value="p.id">
                  {{ p.name }}
                </option>
              </select>
            </label>
            <label
              >Nama varian
              <input
                v-model="variantForm.name"
                type="text"
                placeholder="mis. 250gr"
            /></label>
            <label
              >SKU (kosongkan untuk otomatis)
              <input v-model="variantForm.sku" type="text"
            /></label>
            <div class="desk-grid">
              <label
                >Harga beli
                <input
                  v-model.number="variantForm.purchase_price"
                  type="number"
                  min="0"
              /></label>
              <label
                >Harga jual
                <input
                  v-model.number="variantForm.selling_price"
                  type="number"
                  min="0"
              /></label>
              <label
                >Stok minimum
                <input
                  v-model.number="variantForm.minimum_stock"
                  type="number"
                  min="0"
              /></label>
              <label
                >Stok awal
                <input
                  v-model.number="variantForm.initial_stock"
                  type="number"
                  min="0"
              /></label>
            </div>
            <button class="primary-button" type="submit" :disabled="deskBusy">
              Buat varian
            </button>
          </form>
        </div>
      </div>
    </Teleport>
  </section>
</template>

<style scoped>
.desk-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9999;
  display: grid;
  place-items: center;
  background: rgba(20, 30, 28, 0.55);
}
.desk-modal {
  width: min(520px, 92vw);
  max-height: 88vh;
  overflow: auto;
  padding: 20px;
  border-radius: 14px;
  background: #fffaf0;
  color: #2b3b38;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.35);
}
.desk-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}
.desk-tabs {
  display: flex;
  gap: 6px;
  margin-bottom: 14px;
}
.desk-tabs button {
  flex: 1;
  padding: 8px 10px;
  border: 1px solid #315e55;
  border-radius: 8px;
  background: transparent;
  color: #315e55;
  cursor: pointer;
}
.desk-tabs button.active {
  background: #315e55;
  color: #fff3d6;
}
.desk-form {
  display: grid;
  gap: 12px;
}
.desk-form label {
  display: grid;
  gap: 4px;
  font-size: 0.85rem;
  font-weight: 600;
}
.desk-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}
.desk-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.desk-chips span {
  padding: 4px 10px;
  border-radius: 999px;
  background: #e3ebe4;
  font-size: 0.8rem;
}
.desk-empty {
  margin: 0;
  font-size: 0.85rem;
  opacity: 0.7;
}
</style>