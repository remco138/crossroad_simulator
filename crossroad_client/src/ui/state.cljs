(ns ui.state
  (:require [cljsjs.paperjs]
            [reagent.core :as r :refer [atom]]
            [cljs.core.async :as async :refer [chan close! dropping-buffer sliding-buffer]]))

(declare state)
(declare sensors-chan)
;example state
(defn init! []
  (def state (atom {:roads {"road1" {:path (js/paper.Path. "M 0.65134816,139.3758 C 212.96155,173.64748 449.60053,223.39674 454.35934,176.35395 461.74642,103.32967 481.01948,3.7561552 481.01948,3.7561552") :light 1}
                            "road2" {:path (js/paper.Path. "M 2.5808761,151.33359 707.29671,268.8545") :light 2}
                            "road3" {:path (js/paper.Path. "M -3.767145,159.39796 705.44943,278.24318") :light 3}
                            "road4" {:path (js/paper.Path. "M -13.143726,172.741 C 464.44011,272.77115 400.8466,168.08815 365.55987,456.15259") :light 4}
                            "road5" {:path (js/paper.Path. "M 393.88892,460.94054 C 413.2146,336.85535 456.72085,190.66965 372.8835,159.5339 339.61396,147.17816 4.1074141,97.164591 4.1074141,97.164591") :light 5}
                            "road6" {:path (js/paper.Path. "M 410.74143,454.50962 479.74599,3.5155303") :light 6}}
                    :traffic-lights {
                                     0 {:point (js/paper.Path.Circle. (js/paper.Point. 0 0) 4)}
                                     1 {:point (js/paper.Path.Circle. (js/paper.Point. 372 194) 4)}
                                     2 {:point (js/paper.Path.Circle. (js/paper.Point. 369 212) 4)}
                                     3 {:point (js/paper.Path.Circle. (js/paper.Point. 367 221) 4)}
                                     4 {:point (js/paper.Path.Circle. (js/paper.Point. 363 242) 4)}
                                     5 {:point (js/paper.Path.Circle. (js/paper.Point. 422 273) 4)}
                                     6 {:point (js/paper.Path.Circle. (js/paper.Point. 438 276 ) 4)}
                                     }
                    :sensors {
                              0 {:point (js/paper.Path.Circle. (js/paper.Point. 0 0) 4) :chan (chan (dropping-buffer 1))}
                              1 {:point (js/paper.Path.Circle. (js/paper.Point. 294 186) 4) :chan (chan (dropping-buffer 1))}
                              2 {:point (js/paper.Path.Circle. (js/paper.Point. 293 199) 4) :chan (chan (dropping-buffer 1))}
                              3 {:point (js/paper.Path.Circle. (js/paper.Point. 292 211) 4) :chan (chan (dropping-buffer 1))}
                              4 {:point (js/paper.Path.Circle. (js/paper.Point. 292 224) 4) :chan (chan (dropping-buffer 1))}
                              5 {:point (js/paper.Path.Circle. (js/paper.Point. 417 323) 4) :chan (chan (dropping-buffer 1))}
                              6 {:point (js/paper.Path.Circle. (js/paper.Point. 430 325 ) 4) :chan (chan (dropping-buffer 1))}
                              }}))

  (comment (def sensors-chan (atom (async/merge (map #(-> % val :chan) (:sensors @state)))))))


(def ui-state (r/atom {:speed 3 :sensor-refresh 1000 :last-packed "last-packet"}))

(def cars (atom []))
(def lights (atom [0 1 0 0 2 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1]))

;(add-car-channel! {:test "a"})
(defn add-car-channel! [car]
  (let [new-car (assoc car :chan (chan 10))]
    (swap! cars conj new-car)
    new-car))

