(ns ui.state
  (:require [cljsjs.paperjs]
            [reagent.core :as r :refer [atom]]
            [cljs.core.async :as async :refer [chan close! dropping-buffer sliding-buffer]]))

(declare state)
(declare sensors-chan)
;example state
(defn init! []
  (def state (atom {:roads {"road0" {:path (js/paper.Path. "M -13.143726,172.741 C 464.44011,272.77115 400.8466,168.08815 365.55987,456.15259") :light 0}
                            "road1" {:path (js/paper.Path. "M -4.9288972,162.88321 697.30508,278.85324 965.99691,293.64823") :light 1}
                            "road2" {:path (js/paper.Path. "M 393.49029,454.50962 462.49485,3.5155303") :light 2}
                            "road3" {:path (js/paper.Path. "M -2.3235043,191.89526 C 441.78778,275.68235 414.93451,239.97297 663.84169,290.02204 c 103.8881,20.88932 298.71124,12.43899 298.71124,12.43899") :light 3}}
                    :traffic-lights {
                                     0 {:point (js/paper.Path.Circle. (js/paper.Point. 363 242) 4)}
                                     1 {:point (js/paper.Path.Circle. (js/paper.Point. 366 224) 4)}
                                     2 {:point (js/paper.Path.Circle. (js/paper.Point. 363 250) 4)}
                                     3 {:point (js/paper.Path.Circle. (js/paper.Point. 422 273) 4)}}
                    :sensors {
                              0 {:point (js/paper.Path.Circle. (js/paper.Point. 290 224) 4) :chan (chan (dropping-buffer 1))}
                              1 {:point (js/paper.Path.Circle. (js/paper.Point. 292 211) 4) :chan (chan (dropping-buffer 1))}
                              2 {:point (js/paper.Path.Circle. (js/paper.Point. 413 325) 4) :chan (chan (dropping-buffer 1))}
                              3 {:point (js/paper.Path.Circle. (js/paper.Point. 289 241) 4) :chan (chan (dropping-buffer 1))}}}))

  (comment (def sensors-chan (atom (async/merge (map #(-> % val :chan) (:sensors @state)))))))


(def ui-state (r/atom {:speed 3 :sensor-refresh 1000}))

(def cars (atom []))
(def lights (atom [0 1 1 2]))

;(add-car-channel! {:test "a"})
(defn add-car-channel! [car]
  (let [new-car (assoc car :chan (chan 10))]
    (swap! cars conj new-car)
    new-car))

