# 学んだこと

関心の分離
疎結合
コマンドパターン
引数は最小限
YAGNI(You aren't gonna need it) 今必要ないなら将来必要かもしれなくても実装しなくてよい
DRY(Don't repeat yourself)
Engine のようなオーケストレーターはユニットテストとインテグレーションテストを組み合わせる
ライフタイムは引数
New Type Pattern
引数は基本的に&T、Copy 可能な場合は T(所有型)
推測するな、計測しろ

# オリジナルの構造

InputOutputMap::addUniverse
InputOutputMap::startUniverses

MasterTimerPrivate::run
MasterTimer::timerTick
emit tickReady
Universe::tick //セマフォをリリース

Universe::run //Universe は QThread を継承しているので QThread::start()で自動的に呼ばれる
Universe::processFaders
Universe::dumpOutput
OutputPatch::dump
QLCIOPlugin::writeUniverse
ARtNetPlugin::writeUniverse
ArtNetController::sendDmx
QUdpSocket::writeDatagram
