(ns ui.sensors
  (:require
   [ui.state :as state]
   [ui.network :as network]
   [cljs.core.async :as async :refer [chan close! >! <! alts! timeout]])
  (:require-macros
   [cljs.core.async.macros :refer [go-loop alt!]]))


(defn conj-when-unique [xs x]
  (if (:bezet x)
    (conj (remove #(== (:id %) (:id x)) xs) x)
    (if (some #(== (:id x) (:id %)) xs)
      xs
      (conj xs x)))



  )

;j   (conj-when-unique [ {:id 1 :bezet false}] {:id 1 :bezet true})

(defn track-sensors! []
  (print "track!")
  (let [sensors (async/merge (map #(-> % val :chan) (:sensors @state/state)))]
    (go-loop []
             ;drain ch
             (loop [result #{}
                    t (timeout (:sensor-refresh @state/ui-state))]

               (if-let [v (first (alts! [sensors t]))]
                 (recur (conj-when-unique result v) t)
                 (do (when-not (empty? result) (network/send-sensor-states! result)))))

             (recur))))
