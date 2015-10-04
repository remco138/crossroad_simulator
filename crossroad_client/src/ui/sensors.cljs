(ns ui.sensors
  (:require
   [ui.state :as state]
   [cljs.core.async :as async :refer [chan close! >! <!]])
  (:require-macros
   [cljs.core.async.macros :refer [go-loop]]))


(defn track-sensors! []
  (print "track!")
  (go-loop []
    (print "sensor was hit: " (<! @state/sensors-chan))
    (recur)))
