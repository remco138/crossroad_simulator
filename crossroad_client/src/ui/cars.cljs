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
        car (js/paper.Path.Circle. (.getPointAt (:path road) 0) 5)]
    (set! (.-strokeColor car) "black")
    (set! (.-fillColor car) "black")
    {:car car :road road :speed 1 :ahead 14 :dist 6}))

(defn random-walker [roads]
  (let [road (pick-random-road roads)
        car (js/paper.Path.Circle. (.getPointAt (:path road) 0) 7)]
    (set! (.-strokeColor car) "black")
    {:car car :road road :speed 0.1 :ahead 2 :dist 5}))

(defn random-cyclist [roads]
  (let [road (pick-random-road roads)
        car (js/paper.Path.Circle. (.getPointAt (:path road) 0) 2)]
    (set! (.-strokeColor car) "green")
    (set! (.-fillColor car) "green")
    {:car car :road road :speed 0.3 :ahead 4 :dist 2}))

(defn random-bus [roads]
  (let [road (pick-random-road roads)
        car (js/paper.Path.Rectangle. (.getPointAt (:path road) 0) 17)]
    (set! (.-strokeColor car) "black")
    {:car car :road road :speed 0.7 :ahead 18 :dist 5}))

;----------
;Car AI
;----------
(defn destroy-car! [ch car id]
  (reset! state/cars (remove #(identical? car %) @state/cars))
  (.remove (:car car))
  (swap! state/cars-location-ahead dissoc id)
  (close! ch))



(defn trigger-sensors! [car sensor-list]
  (when (-> car :road :light)
    (let [indexes (-> car :road :light)
             sensors (map sensor-list indexes)
             state (map #(.contains (:car car) (.-position (:point %))) sensors)]
         (go (>! (:chan (first sensors)) {:bezet (first state) :id (first indexes)})))))



;(when-not (empty? (vals @state/cars-location-ahead))
;                                       (< 2 (count (filter #(identical? true %)
;                                                           (map #(.contains (:car car) %) (vals @state/cars-location-ahead))))))


(defn may-move? [car lights-state sensor-list dangerous-cars future-position]
  (not  (some #(= false %) (map
                            (fn [x] (let [light-index x
                                     light-state (nth lights-state light-index)
                                     sensor (:point (sensor-list light-index))]
                                 (not
                                  (or

                                  ;collision check
                                   (when-not (or (empty? dangerous-cars) (nil? future-position))
                                     (<= 1 (count (filter (fn [x] (> (:dist car) (.-length (.subtract future-position x)))) dangerous-cars))))

                                   (and
                                    ;the traffic light allows us to move (green/orange => 1, 2)
                                    (== light-state 0)
                                    ;we are not standing infront of the traffic light (by checking .contains of its sensor)
                                    (.contains (:car car) (.-position sensor)))))))

                              (-> car :road :light)))))


(defn find-dangerous-cars [current-car cars]
  (when (<= 2 (count cars))
     (filter #(> 100 (.-length (.subtract  (.-position (:car current-car))  %))) cars)))


(defn car-ai [ch car x]
  (let [id (rand-int 99999)
        perf-rate 15]
    (go-loop [x 0
              tick 15
              dangerous-cars []]
             (let [data (<! ch)
                   path (:path (:road car))
                   sensors (:sensors @state/state)
                   x-add (* (:speed car) (:speed @state/ui-state))
                   may-move (may-move? car @state/lights sensors dangerous-cars (.getPointAt path (+ x (:ahead car))))]
               (cond
                ;end of life?
                (>= x (.-length path))
                (destroy-car! ch car id)

                ;update dangerous cars, for perf reasons we can't check every car for collision
                (== 0 (mod tick perf-rate))
                (recur x (inc tick) (find-dangerous-cars car (vals @state/cars-location-ahead)))

                ;are we allowed to move? traffic light? car infront? intersecting car nearby?
                may-move
                (do

                  (set! (.-position (:car car)) (.getPointAt path x))
                  (trigger-sensors! car sensors)
                  (swap! state/cars-location-ahead assoc id  (.getPointAt path x))
                  (recur (+ x x-add) (inc tick) dangerous-cars))

                :default
                (do
                  (trigger-sensors! car sensors)
                  (swap! state/cars-location-ahead assoc id  (.getPointAt path  x))
                  (recur x (inc tick) dangerous-cars))
                )))))
