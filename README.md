# Pitch Controller

ゲームコントローラーなどのjoystickを利用してmidi信号にピッチベンドを適用できます.

[![Pitch Controller Demo](https://img.youtube.com/vi/xuWjXmqUC6k/0.jpg)](https://youtu.be/xuWjXmqUC6k)

## 対応環境

NixがインストールされたLinux

## Usage

1. コントローラーをPCに接続
2. nix run github:khimoo/pitch_controller
3. 仮想のMIDIクライアントが立ち上がります。接続方法は2通りに対応しています。
	 - 並列接続 (DAWが両者を受け取る設定):
		 - DAWソフトの入力に直接**MIDIキーボード**と**本ソフトの仮想クライアント**の両方を接続します。
		 - DAW側で両方の入力を受け取りつつ、本ソフトが送るピッチベンドを適用できます。
	 - 直列接続 (キーボード → 仮想クライアント → DAW):
		 - **MIDIキーボード**を一度本ソフトの仮想クライアントに接続し、仮想クライアントからDAWへ接続します。
		 - この接続では本ソフトがキーボード入力を受け取り、処理したMIDI（ピッチベンド含む）をDAWに送ります。

	 ```text
	 並列接続 (Parallel):
	 [MIDI Keyboard] ---> [DAW]
	 [Pitch Controller] ---> [DAW]

	 直列接続 (Serial):
	 [MIDI Keyboard] ---> [Pitch Controller] ---> [DAW]
	 ```

## Features

- [ ] Gui setting tool
- [ ] Keyconfig
