import QtQuick 2.2
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.0

ApplicationWindow {
    title: getTitle()
    visible: true
    width: 350
    height: 50

    Slider {
        id: slider
        anchors.centerIn: parent
        width: parent.width - 50
        minimumValue: 0
        maximumValue: 100
        stepSize: 1
        value: getVolume()
        onValueChanged: setVolume(value)
    }

    function getVolume() {
        return volume.get_volume();
    }
    function setVolume(val) {
        volume.set_volume(val)
    }
    function getTitle() {
        return volume.get_title();
    }
}
