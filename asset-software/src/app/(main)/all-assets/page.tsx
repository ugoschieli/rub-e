import AssetCard from "@/components/asset-card"

const assets = [
  { title: "Character_Hero_01", type: "FBX Model", image: "/STG_02.png", meta: ["12,450 tris", "3.2 MB", "Character", "Hero"] },
  { title: "Character_Enemy_Orc", type: "FBX Model", image: "/STG_02.png", meta: ["18,230 tris", "4.5 MB", "Character", "Enemy"] },
  { title: "Metal_Rust_PBR", type: "Texture 2K", image: "/STG_02.png", meta: ["2048x2048", "8.4 MB", "PBR", "Metal"] },
  { title: "Wood_Floor_Material", type: "Material", image: "/STG_02.png", meta: ["1.2 MB", "Wood", "Floor"] },
  { title: "Stone_Wall_Tileable", type: "Texture 4K", image: "/STG_02.png", meta: ["4096x4096", "15.8 MB", "Stone", "Wall"] },
  { title: "Ambient_Forest", type: "Audio WAV", image: "/STG_02.png", meta: ["4.5 MB", "2:34", "Ambient", "Nature"] },
  { title: "Sky_Sunset_HDRI", type: "Texture HDRI", image: "/STG_02.png", meta: ["8192x4096", "12.3 MB", "HDRI", "Sky"] },
  { title: "Weapon_Sword_Iron", type: "FBX Model", image: "/STG_02.png", meta: ["6,780 tris", "1.8 MB", "Weapon", "Sword"] },
  { title: "Explosion_SFX", type: "Audio WAV", image: "/STG_02.png", meta: ["2.1 MB", "0:15", "Effect", "Explosion"] },
  { title: "Grass_Tileable", type: "Texture 2K", image: "/STG_02.png", meta: ["2048x2048", "3.6 MB", "Nature", "Grass"] },
  { title: "Water_Shader_Material", type: "Material", image: "/STG_02.png", meta: ["2.3 MB", "Shader", "Water"] },
  { title: "Character_NPC_Villager", type: "FBX Model", image: "/STG_02.png", meta: ["8,900 tris", "2.7 MB", "Character", "NPC"] },
  { title: "Rock_Cliff_Large", type: "FBX Model", image: "/STG_02.png", meta: ["14,500 tris", "5.1 MB", "Environment", "Rock"] },
  { title: "Fire_SFX", type: "Audio WAV", image: "/STG_02.png", meta: ["3.0 MB", "0:25", "Effect", "Fire"] },
  { title: "Leather_Armor_Material", type: "Material", image: "/STG_02.png", meta: ["1.5 MB", "Armor", "Leather"] }
];

export default function Page() {
  return (
    <div className="flex flex-wrap p-4">
        {
          assets.map((asset, index) => (
            <AssetCard key={index} asset={asset}/>
          ))
        }
    </div>
  )
}
