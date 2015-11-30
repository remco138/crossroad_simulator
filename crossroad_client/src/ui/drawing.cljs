(ns ui.drawing
  (:require
   [ui.state :as state]
   [cljs.core.async :as async :refer [chan close! >! <!]]
   [cljsjs.paperjs])
  (:require-macros
   [cljs.core.async.macros :refer [go]]))

(enable-console-print!)

;display the lines (by setting strokeColor)
(defn init! []
  (doseq [p (:roads @state/state)]
    (-> p val :path
        (-> (.-strokeColor) (set! "black") )))
  (doseq [p (:bus-roads @state/state)]
    (-> p val :path
        (-> (.-strokeColor) (set! "red") )))
  (doseq [p (:cycling-roads @state/state)]
    (-> p val :path
        (-> (.-strokeColor) (set! "green") )))
  (doseq [p (:pedestrian-roads @state/state)]
    (-> p val :path
        (-> (.-strokeColor) (set! "orange") )))

  (doseq [p (:traffic-lights @state/state)]
    (-> p val :point
        (-> (.-fillColor) (set! "orange") )))

  (doseq [p (:sensors @state/state)]
    (-> p val :point
        (-> (.-fillColor) (set! "teal"))))


    (doseq [p (:sensors @state/state)]
      (-> p val :point
          (-> (.-opacity) (set! 0)))))


(defn state->color [code]
  (case code
    0 "red"
    1 "orange"
    2 "green"
    :default "purple"))

(defn on-frame []
  (go
   (doseq [c @state/cars]
     (>! (:chan c) {:options "nothing.."})))

  (doseq [[k v] (:traffic-lights @state/state)]
    (-> v :point
        (-> (.-fillColor) (set! (state->color (@state/lights k))))))

  (if (:display-sensors @state/ui-state)
    (doseq [p (:sensors @state/state)]
      (-> p val :point
          (-> (.-opacity) (set! 1))))
    (doseq [p (:sensors @state/state)]
      (-> p val :point
          (-> (.-opacity) (set! 0)))))


    (doseq [[k v] (:traffic-lights @state/state)]
       (-> v :point
           (-> (.-fillColor) (set! (state->color (@state/lights k))))))
  )
