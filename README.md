# Pitch Controller

ゲームコントローラーなどのjoystickを利用してmidi信号にピッチベンドを適用できます.

[![Pitch Controller Demo](https://img.youtube.com/vi/xuWjXmqUC6k/0.jpg)](https://youtu.be/xuWjXmqUC6k)

## 対応環境

NixがインストールされたLinux

## Usage

1. コントローラーをPCに接続
2. nix run github:khimoo/pitch_controller
3. 仮想のmidiクライアントが立ち上がるので, qjackctl的なソフトウェアでmidiキーボードなどと接続させる

## Features

- [ ] Gui setting tool
- [ ] Keyconfig
