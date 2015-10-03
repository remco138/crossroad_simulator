(ns ui.cars
  (:require [cljsjs.paperjs]))


(defn- pick-random-road [roads]
  (let [kv (rand-nth (seq roads))]
    (merge (second kv) {:name (first kv)})))


;(random-car {:road1 {:path (js/paper.Path. "M -13.143726,172.741 C 464.44011,272.77115 400.8466,168.08815 365.55987,456.15259")}
;             :road2 {:path (js/paper.Path. "M -4.9288972,162.88321 697.30508,278.85324 965.99691,293.64823")}})


(defn random-car [roads]
  (let [road (pick-random-road roads)
        car (js/paper.Path.Circle. (.getPointAt (:path road) 5) 5)]
    (set! (.-strokeColor car) "black")
    {:car car :road road}))
