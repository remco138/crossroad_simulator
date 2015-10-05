(ns ui.sensors
  (:require
   [ui.state :as state]
   [ui.network :as network]
   [cljs.core.async :as async :refer [chan close! >! <! alts! timeout]]
   [clojure.data.json])
  (:require-macros
   [cljs.core.async.macros :refer [go-loop alt!]]))


(defn track-sensors! []
  (print "track!")
  (let [sensors (map #(-> % val :chan) (:sensors @state/state))]
    (go-loop []
             (print "sensor was hit: " ;even though we use a dropping buffer of 1.., 2 are being <!'d
              (first (alts! sensors))) ;might aswel print the duplicate <!

             (network/send! (to-json {:id (first (alts! sensors))}))
             (<! (timeout 200))

             (recur))))
