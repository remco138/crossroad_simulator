(ns ui.cars
  (:require
   [ui.state :as state]
   [cljsjs.paperjs]
   [cljs.core.async :as async :refer [chan close! >! <! put!]])
  (:require-macros
   [cljs.core.async.macros :refer [go-loop go]]))
(enable-console-print!)

;----------
;Code related to car spawning
;----------
(defn- pick-random-road [roads]
  (let [kv (rand-nth (seq roads))]
    (merge (second kv) {:name (first kv)})))


;(random-car {:road1 {:path (js/paper.Path. "M -13.143726,172.741 C 464.44011,272.77115 400.8466,168.08815 365.55987,456.15259")}
;             :road2 {:path (js/paper.Path. "M -4.9288972,162.88321 697.30508,278.85324 965.99691,293.64823")}})
(defn random-car [roads]
  (let [road (pick-random-road roads)
        car (js/paper.Path.Circle. (.getPointAt (:path road) 0) 7)]
    (set! (.-strokeColor car) "black")
    {:car car :road road :speed 1}))

(defn random-walker [roads]
  (let [road (pick-random-road roads)
        car (js/paper.Path.Circle. (.getPointAt (:path road) 0) 7)]
    (set! (.-strokeColor car) "black")
    {:car car :road road :speed 0.1}))

(defn random-cyclist [roads]
  (let [road (pick-random-road roads)
        car (js/paper.Path.Circle. (.getPointAt (:path road) 0) 2)]
    (set! (.-strokeColor car) "black")
    {:car car :road road :speed 0.3}))

(defn random-bus [roads]
  (let [road (pick-random-road roads)
        car (js/paper.Path.Rectangle. (.getPointAt (:path road) 0) 17)]
    (set! (.-strokeColor car) "black")
    {:car car :road road :speed 0.7}))

;----------
;Car AI
;----------
(defn destroy-car! [ch car]
  (reset! state/cars (remove #(identical? car %) @state/cars))
  (.remove (:car car))
  (close! ch))



(defn trigger-sensors! [car sensor-list]
  (let [index (-> car :road :light)
        sensor (sensor-list index)
        state (.contains (:car car) (.-position (:point sensor)))]
    (go (>! (:chan sensor) {:bezet state :id index}))))


(defn may-move? [car lights-state sensor-list]
  (not  (some #(= false %) (map
                              #(let [light-index %
                                     light-state (nth lights-state light-index)
                                     sensor (:point (sensor-list light-index))]
                                 (not (and
                                       ;the traffic light allows us to move (green/orange => 1, 2)
                                       (== light-state 0)
                                       ;we are not standing infront of the traffic light (by checking .contains of its sensor)
                                       (.contains (:car car) (.-position sensor)))))

                              (-> car :road :light)))))



(defn car-ai [ch car x]
  (go-loop [x 0]
           (let [data (<! ch)
                 path (:path (:road car))
                 sensors (:sensors @state/state)]
             (cond
              ;end of life?
              (>= x (.-length path))
              (destroy-car! ch car)

              ;are we allowed to move? traffic light? car infront? intersecting car nearby?
              (may-move? car @state/lights sensors)
              (do
                (set! (.-position (:car car)) (.getPointAt path x))
                ;(trigger-sensors! car sensors)
                (recur (+ x (* (:speed car) (:speed @state/ui-state)))))

              :default
              (do
                ;(trigger-sensors! car sensors)
                (recur x))
              ))))



;todo:
;pass state of other cars
;filter cars with roads that intersect (include own road)
;check if overlap with getPointAt(tick + magicdistance)
;->halt
