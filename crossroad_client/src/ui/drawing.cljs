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
  (def path1 (js/paper.Path. "M -13.143726,172.741 C 464.44011,272.77115 400.8466,168.08815 365.55987,456.15259"))
  (def path2 (js/paper.Path. "M -4.9288972,162.88321 697.30508,278.85324 965.99691,293.64823"))
  (def path3 (js/paper.Path. "M 393.49029,454.50962 462.49485,3.5155303"))
  (def path4 (js/paper.Path. "M -2.3235043,191.89526 C 441.78778,275.68235 414.93451,239.97297 663.84169,290.02204 c 103.8881,20.88932 298.71124,12.43899 298.71124,12.43899"))
  (set! (.-strokeColor path1) "black")
  (set! (.-strokeColor path2) "black")
  (set! (.-strokeColor path3) "black")
  (set! (.-strokeColor path4) "black"))

(defn on-frame []
  (go
   (doseq [c @state/cars]
     (>! (:chan c) {:multiplier 0.1}))))
