# QLC+の改善点
- 学習コストが高いくせに慣れても操作性・生産性低い
- デバッグしづらい
- バグが多い
- 操作回数が無駄に多い

## 具体的に
- モジュール化(小さい関数)
- ファンクションの再利用→他ファイルからのインポート？
- シーンエディタでのコピペ
- スライダーいらない？
- Tabキーで移動
- RGBスクリプトの改善(型サポート、ergonomic)
- キーボードショートカットの充実
- テーブル式のエディタ(Excelライク？)

## チェイサーのデメリット
- 柔軟性低い？
- 一覧性が低い、直感的でない
## タイムラインのデメリット
細かい時間指定ができない
ループを手で置かないといけない

=> **タイムラインを改善していくのがよさげ**

# Tsukuyomiについて
## コンポーネント
- ### エンジン
Rust  
C#  
- ### エディタ
Slint  
Unity  
~~Avalonia~~  
Excel?  
- ### プレビュー
Unity  
~~wGPU~~  
Capture  
three-d 

## 構成案
### Rustエンジン
高いリアルタイム性、安全性
- #### R1: Rustエンジン+Slintエディタ+Captureプレビュー  
開発期間が短い。  
単一ウィンドウが本当に必要か、別ウィンドウでのプレビューがどの程度不便かわかる。 
Unityにもthree-dにもシームレスに移行可。
- #### R2: Rustエンジン+Slintエディタ+Unityプレビュー  
シンプル。Unityでカスタマイズされたプレビューを見れるが別ウィンドウなのでUXは劣る。
- #### R3: Rustエンジン+Slintエディタ+three-dプレビュー
実装難易度が高め。単一ウィンドウなのでUXは良い。
- ~~#### Rustエンジン+beby_uiエディタ+bevyプレビュー
学習コストが高め。~~
### C#エンジン
- #### C1: C#エンジン+Unityエディタ+Unityプレビュー
シンプル。1ウィンドウで作業できるためUXは良い。  
パフォーマンスはRustに劣るが重大ではない。  
私自身のUnityスキルが低い&C#の安全性は懸念。  
rustaceanよりC#erの方が多そうなので私が抜けた後もメンテナンスしやすい。
~~Unity UI Toolkitがエディタ向けのコンポーネントを持っているかは要調査。~~  
- ~~#### C#エンジン+Avaloniaエディタ+Unityプレビュー  
Unity UI ToolKitよりデスクトップアプリ向け。  
Unityより学習コストが低い(プレビュー部分だけならUnity使いに簡単に委託できる)。  
AvaloniaとUnityを統合するのにIPCを使うのでパフォーマンスは懸念(致命的?)。~~

# 今後の予定
Unity UI Toolkitとthree-dを触ってみる。Slintで簡単なエディタを試作する。
# QLC+の改善点
- 学習コストが高いくせに慣れても操作性・生産性低い
- デバッグしづらい
- バグが多い
- 操作回数が無駄に多い

## 具体的に
- モジュール化(小さい関数)
- ファンクションの再利用→他ファイルからのインポート？
- シーンエディタでのコピペ
- スライダーいらない？
- Tabキーで移動
- RGBスクリプトの改善(型サポート、ergonomic)
- キーボードショートカットの充実
- テーブル式のエディタ(Excelライク？)

## チェイサーのデメリット
- 柔軟性低い？
- 一覧性が低い、直感的でない
## タイムラインのデメリット
細かい時間指定ができない
ループを手で置かないといけない

=> **タイムラインを改善していくのがよさげ**

# Tsukuyomiについて
## コンポーネント
- エンジン
Rust
C#
- エディタ
Slint
Unity
Avalonia
Excel?
- プレビュー
Unity
wGPU
Capture

## 構成案
### Rustエンジン
高いリアルタイム性、安全性
- #### R1: Rustエンジン+Slintエディタ+Captureプレビュー  
開発期間が短い。  
単一ウィンドウが本当に必要か、別ウィンドウでのプレビューがどの程度不便かわかる。 
Unityにもthree-dにもシームレスに移行可。
- #### R2: Rustエンジン+Slintエディタ+Unityプレビュー  
シンプル。Unityでカスタマイズされたプレビューを見れるが別ウィンドウなのでUXは劣る。
- #### R3: Rustエンジン+Slintエディタ+three-dプレビュー
実装難易度が高め。単一ウィンドウなのでUXは良い。
- ~~#### Rustエンジン+beby_uiエディタ+bevyプレビュー
学習コストが高め。~~
### C#エンジン
- #### C1: C#エンジン+Unityエディタ+Unityプレビュー
シンプル。1ウィンドウで作業できるためUXは良い。  
パフォーマンスはRustに劣るが重大ではない。  
私自身のUnityスキルが低い&C#の安全性は懸念。  
rustaceanよりC#erの方が多そうなので私が抜けた後もメンテナンスしやすい。
~~Unity UI Toolkitがエディタ向けのコンポーネントを持っているかは要調査。~~  
- ~~#### C#エンジン+Avaloniaエディタ+Unityプレビュー  
Unity UI ToolKitよりデスクトップアプリ向け。  
Unityより学習コストが低い(プレビュー部分だけならUnity使いに簡単に委託できる)。  
AvaloniaとUnityを統合するのにIPCを使うのでパフォーマンスは懸念(致命的?)。~~

## プロトタイピング
### Rust
#### Slintとthree-dの統合方法
- ##### three-dでレンダリング->`Vec<u8>`に変換->Slintの`Image`に変換
いわゆるオフスクリーン？簡単だがパフォーマンスがやや落ちる。
<details>
<summary>コード例</summary>
```rust
use three_d::*;
    2
    3 fn render_scene_to_pixels() -> Vec<u8> {
    4     // 1. ウィンドウなしでthree-dのコンテキストを作成
    5     let context = HeadlessContext::new().unwrap();
    6
    7     // 2. 3Dオブジェクトを定義
    8     let mut camera = Camera::new_perspective(...);
    9     let mut cube = Gm::new(
   10         Mesh::new(&context, &CpuMesh::cube()),
   11         PhysicalMaterial::new_opaque(&context, &CpuMaterial::default()),
   12     );
   13     let light = AmbientLight::new(&context, 0.5, Srgba::WHITE);
   14
   15     // 3. オフスクリーンのテクスチャにレンダリング
   16     let width = 512;
   17     let height = 512;
   18     let mut target = RenderTarget::new_color(&context, width, height).unwrap();
   19     target.render(&camera, &cube, &[&light]);
   20
   21     // 4. レンダリング結果をCPUメモリにピクセルデータとして読み出す
   22     let pixels: Vec<Srgba> = target.read_color().unwrap();
   23
   24     // `Srgba`のVecを、Slintが要求する`u8`のVecに変換する
   25     let byte_pixels: Vec<u8> = pixels.iter().flat_map(|p| p.to_rgba_u8()).collect();
   26
   27     byte_pixels
   28 }
   29
   30 // main.rs や engine.rs の中で
   31 fn update_preview_in_slint(app: &App) {
   32     let pixel_data = render_scene_to_pixels();
   33
   34     let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
   35         &pixel_data,
   36         512, // width
   37         512, // height
   38     );
   39
   40     let slint_image = slint::Image::from_rgba8(buffer);
   41     app.set_preview_image(slint_image);
   42 }
```
</details>

- ##### [公式](https://github.com/slint-ui/slint/tree/master/examples/bevy)の真似
実装難易度が高い。必要な知識が重い。
### C#