(ns ui.state
  (:require [cljsjs.paperjs]
             [cljs.core.async :as async :refer [chan close!]]))

;example state
(def state (atom {:roads {:road1 {:path (js/paper.Path. "M -13.143726,172.741 C 464.44011,272.77115 400.8466,168.08815 365.55987,456.15259") :traffic-light "light1", :intersections [[1,1] [666,666] [999,999]]}
                          :road2 {:path (js/paper.Path. "M -4.9288972,162.88321 697.30508,278.85324 965.99691,293.64823") :traffic-light "light2", :intersections [[444,444]]}
                          :road3 {:path (js/paper.Path. "M 393.49029,454.50962 462.49485,3.5155303") :traffic-light "light2", :intersections [[444,444]]}
                          :road4 {:path (js/paper.Path. "M -2.3235043,191.89526 C 441.78778,275.68235 414.93451,239.97297 663.84169,290.02204 c 103.8881,20.88932 298.71124,12.43899 298.71124,12.43899") :traffic-light "light2", :intersections [[444,444]]}}
                  :nothing []}
                  ))


(def car-channels (repeatedly 5 #(chan 1)))
