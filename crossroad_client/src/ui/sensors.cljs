(ns ui.sensors
  (:require
   [ui.state :as state]
   [ui.network :as network]
   [cljs.core.async :as async :refer [chan close! >! <! alts! timeout]])
   ;[clojure.data.json])
  (:require-macros
   [cljs.core.async.macros :refer [go-loop alt!]]))


(defn track-sensors! []
  (print "track!")
  (let [sensors (async/merge (map #(-> % val :chan) (:sensors @state/state)))]
    (go-loop []
             ;drain ch
             (loop [result #{}
                    t (timeout (:sensor-refresh @state/ui-state))]

               (if-let [v (first (alts! [sensors t]))]
                 (recur (conj result v) t)
                 (when-not (empty? result) (print result))))

             (recur))))
