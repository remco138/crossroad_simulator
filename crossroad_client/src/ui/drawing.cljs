(ns ui.drawing
  (:require
   [ui.state :as state]
   [ui.cars :as cars]
   [cljs.core.async :as async :refer [chan close! >! <!]]
   [cljsjs.paperjs])
  (:require-macros
   [cljs.core.async.macros :refer [go]]))

(enable-console-print!)

; this shouldnt be here..
(defn spawn-car! []
  (let [car (state/add-car-channel! (cars/random-car　(:roads @state/state)))]
    (cars/car-ai (:chan car) car 0)))
(defn spawn-walker! []
  (let [car (state/add-car-channel! (cars/random-walker　(:pedestrian-roads @state/state)))]
    (cars/car-ai (:chan car) car 0)))
(defn spawn-bus! []
  (let [car (state/add-car-channel! (cars/random-bus　(:bus-roads @state/state)))]
    (cars/car-ai (:chan car) car 0)))
(defn spawn-cyclist! []
  (let [car (state/add-car-channel! (cars/random-cyclist　(:cycling-roads @state/state)))]
    (cars/car-ai (:chan car) car 0)))

;display the lines (by setting strokeColor)
(defn init! []

  (set! (.-onLoad state/raster) (fn [] (this-as this (do
                                                 (set! this.position js/paper.view.center)
                                                 (set! this.bounds.height js/paper.view.bounds.height)
                                                 (set! this.bounds.width js/paper.view.bounds.width) ))))

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


(defn spawn []
  (let [n (rand-int 100)]
    (cond
     (<= 80 n) (spawn-walker!)
     (<= 30 n) (spawn-car!)

     (<= 2 n) (spawn-cyclist!)
     :default (spawn-bus!)
     )
    (println n)))

(defn state->color [code]
  (case code
    0 "red"
    1 "orange"
    2 "green"
    :default "purple"))

(def tick (atom 0))

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

  (let [grab-roads-fn (juxt :roads :bus-roads :cycling-roads :pedestrian-roads)
        paths (->> (grab-roads-fn @state/state) (mapcat vals) (map :path))]

    (if (:display-paths @state/ui-state)
      (doseq [x paths]
        (-> x (.-opacity) (set! 1)))

      (doseq [x paths]
        (-> x (.-opacity) (set! 0)))))


    (doseq [[k v] (:traffic-lights @state/state)]
       (-> v :point
           (-> (.-fillColor) (set! (state->color (@state/lights k))))))

  (when (> 1 (mod @tick (* 60 (:spawn-rate @state/ui-state))))
    (spawn))

  (swap! tick inc))
