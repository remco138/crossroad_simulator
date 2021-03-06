(ns ui.state
  (:require [cljsjs.paperjs]
            [reagent.core :as r :refer [atom]]
            [cljs.core.async :as async :refer [chan close! dropping-buffer sliding-buffer]]))

(declare state)
(declare sensors-chan)
;example state
; "road1" {:path (js/paper.Path. "M 0.65134816,139.3758 C 212.96155,173.64748 449.60053,223.39674 454.35934,176.35395 461.74642,103.32967 481.01948,3.7561552 481.01948,3.7561552") :light 1}
(defn init! []
   (def raster (js/paper.Raster. "plattegrond2.jpg"))
  (def state (atom {:roads {
                            ;"road0-0"  {:path (js/paper.Path. "m 0,218.74405 310.10976,53.39638 c 0,0 13.96521,0 18.89411,-16.01891 C 334.00915,239.85438 370.8995,1.0511003 370.8995,1.0511003")                         :light [0]}
                            "road0-1"  {:path (js/paper.Path. "m 0,218.74405 318.32459,55.45009 c 0,0 34.17334,7.08454 49.69972,71.05826 11.50091,47.38749 -1.23222,69.00457 -2.46445,112.95389")                         :light [0]}
                            "road1"    {:path (js/paper.Path. "M 0.65134816,139.3758 C 212.96155,173.64748 449.60053,223.39674 454.35934,176.35395 461.74642,103.32967 481.01948,3.7561552 481.01948,3.7561552")          :light [1]}
                            "road2"    {:path (js/paper.Path. "M 2.5808761,151.33359 735.75964,272.33976")                                                                                                                :light [2]}
                            "road3"    {:path (js/paper.Path. "M -3.767145,159.39796 705.44943,278.24318")                                                                                                                :light [3]}
                            "road4"    {:path (js/paper.Path. "M -13.143726,172.741 C 464.44011,272.77115 400.8466,168.08815 365.55987,456.15259")                                                                        :light [4]}
                            "road5"    {:path (js/paper.Path. "M 393.88892,460.94054 C 413.2146,336.85535 457.8826,202.86805 374.04525,171.7323 340.77571,159.37656 2.3647859,109.94386 2.3647859,109.94386")             :light [5]}
                            "road6"    {:path (js/paper.Path. "M 410.74143,454.50962 479.74599,3.5155303")                                                                                                                :light [6]}
                            "road7"    {:path (js/paper.Path. "m 420.22439,458.13118 c 18.23499,-101.75024 24.92157,-187.95697 42.4863,-203.16024 79.6678,-9.56421 192.14692,14.30256 271.18187,28.52618")                :light [7]}
                            "road8"    {:path (js/paper.Path. "M 796.96198,248.3399 478.64189,201.28893 c 0,0 -57.12357,-17.76519 -69.70513,60.41112 -6.6178,41.12013 -29.62469,196.91698 -29.62469,196.91698")           :light [8]}
                            "road9"    {:path (js/paper.Path. "M 799.28548,237.88412 2.3235043,109.51051")                                                                                                                :light [9]}
                            "road10"   {:path (js/paper.Path. "M 802.18986,226.84748 1.1617522,96.731239")                                                                                                                :light [10]}
                            "road11"   {:path (js/paper.Path. "M 803.93249,213.48733 493.16378,162.66068 c 0,0 -37.08922,-15.50908 -26.72029,-80.451343 11.03665,-69.124253 12.77928,-82.48440328 12.77928,-82.48440328") :light [11]}
                            "road12"   {:path (js/paper.Path. "M 449.35111,0.22961744 423.2338,182.41157 c 0,0 -9.62058,35.79156 25.62401,43.004 21.63262,4.42689 308.9601,52.0645 308.9601,52.0645")                     :light [12]}
                            "road13"   {:path (js/paper.Path. "M 440.30407,1.467562 365.37105,458.61703")                                                                                                                 :light [13]}
                            "road14"   {:path (js/paper.Path. "M 431.01005,1.467562 410.67939,110.09139 c 0,0 -4.64701,51.69797 -61.57286,42.40396 C 289.51514,142.76615 2.3235043,95.569487 2.3235043,95.569487")        :light [14]}
                            }
                    :bus-roads {
                                "road15-0" {:path (js/paper.Path. "m 0,191.41404 368.85631,58.37804 c 0,0 30.14868,-1.52207 26.19781,22.80367 -4.4792,27.57867 -29.68306,186.02128 -29.68306,186.02128")                      :light [15]}
                                "road15-1" {:path (js/paper.Path. "M -2.3235043,191.89526 C 441.78778,275.68235 434.6843,232.42158 663.84169,290.02204 c 102.77062,25.83218 298.71124,12.43899 298.71124,12.43899")           :light [15]}
                                "road15-2" {:path (js/paper.Path. "M -1.2322242,190.81363 401.29436,255.71078 c 0,0 49.69972,-8.62557 55.03935,-71.05827 5.6362,-65.90027 23.00152,-183.1906683 23.00152,-183.1906683")       :light [15]}
                                "road16-0" {:path (js/paper.Path. "M 802.17798,200.26069 506.58156,155.46068 c 0,0 -49.34822,-8.041 -39.53938,-72.154551 7.66209,-50.081732 12.70379,-78.9690974 12.70379,-78.9690974")       :light [16]}
                                "road16-1" {:path (js/paper.Path. "M 801.3565,199.84994 488.15734,155.5784 c 0,0 -94.367,11.19213 -122.79483,9.24774 C 318.92774,161.65011 4.0661325,110.67226 4.0661325,110.67226")          :light [16]}
                                "road16-2" {:path (js/paper.Path. "M 801.76724,199.84994 498.64008,153.43616 c 0,0 -67.77234,-20.94781 -90.77386,115.82908 -20.86699,124.08405 -28.34115,188.94105 -28.34115,188.94105")      :light [16]}
                                }
                    :cycling-roads {

                                    "road17" {:path (js/paper.Path. "m 367.11368,6.1145703 c 0,0 -44.72746,226.5416697 -45.88921,246.2914597 -1.16175,19.74978 -21.18133,11.57624 -21.18133,11.57624 L 2.0537071,215.18868") :light [17]}
                                    "road19" {:path (js/paper.Path. "M -0.00140129,221.61203 458.15057,296.01769 c 0,0 7.62678,6.8693 5.99457,20.63938 -1.4789,12.4767 -22.29526,141.96089 -22.29526,141.96089")             :light [19]}
                                    "road19-2" {:path (js/paper.Path. "m -3.3564035,221.27932 462.8350935,75.36465 19.37532,9.36804 c 124.17162,35.79728 258.11533,29.17082 383.15126,24.7235")                              :light [19]}

                                    "road20" {:path (js/paper.Path. "m 449.24323,453.89443 c 0,0 20.81636,-129.48419 22.29526,-141.96089 1.63221,-13.77008 -5.99457,-20.63938 -5.99457,-20.63938 L 7.3919441,216.8885")         :light [20]}
                                    "road20-2" {:path (js/paper.Path. "M 859.1009,323.765 C 734.06497,328.21232 596.636,336.58141 489.30979,304.26938 L 457.73607,289.67346 -5.0990317,214.30881")                              :light [20]}

                                    "road22" {:path (js/paper.Path. "m 450.17896,457.74572 26.7203,-146.67121 c 0,0 32.70273,8.7046 36.88562,-15.68365 2.67206,-15.5794 49.95535,-294.79460861 49.95535,-294.79460861")                                                :light [22]}
                                    "road22-2" {:path (js/paper.Path. "m 858.53484,323.27291 c 0,0 -253.97818,15.90967 -325.53121,-10.82273 -24.63702,-9.20445 -20.64174,-6.97301 -16.28517,-33.69331 9.08236,-55.70516 46.15045,-279.03193248 46.15045,-279.03193248") :light [22]}

                                    "road21" {:path (js/paper.Path. "m 555.5254,-1.8681971 c 0,0 -47.28329,279.2152071 -49.95535,294.7946071 -4.18289,24.38825 -36.88562,15.68365 -36.88562,15.68365 l -26.7203,146.67121")                                    :light [21]}
                                    "road21-2" {:path (js/paper.Path. "m 553.42186,-1.5072867 c 0,0 -34.1929,219.6301067 -43.27526,275.3352667 -4.35657,26.7203 -17.52913,33.73172 7.84182,40.66132 120.43126,32.89352 340.95716,15.76622 340.95716,15.76622") :light [21]}
                                    }
                    :pedestrian-roads {
                                    "road17" {:path (js/paper.Path. "m 367.11368,6.1145703 c 0,0 -44.72746,226.5416697 -45.88921,246.2914597 -1.16175,19.74978 -21.18133,11.57624 -21.18133,11.57624 L 2.0537071,215.18868")  :light [23 26]}
                                    "road19-3" {:path (js/paper.Path. "M -0.00140129,221.61203 458.15057,296.01769 c 0,0 7.62678,6.8693 5.99457,20.63938 -1.4789,12.4767 -22.29526,141.96089 -22.29526,141.96089")             :light [27 29]}
                                    "road19-4" {:path (js/paper.Path. "m -3.3564035,221.27932 462.8350935,75.36465 19.37532,9.36804 c 124.17162,35.79728 258.11533,29.17082 383.15126,24.7235")                              :light [27 29]}
                                    "road20" {:path (js/paper.Path. "m 449.24323,453.89443 c 0,0 20.81636,-129.48419 22.29526,-141.96089 1.63221,-13.77008 -5.99457,-20.63938 -5.99457,-20.63938 L 7.3919441,216.8885")         :light [30 28]}
                                    "road20-2" {:path (js/paper.Path. "M 859.1009,323.765 C 734.06497,328.21232 596.636,336.58141 489.30979,304.26938 L 457.73607,289.67346 -5.0990317,214.30881")                              :light [30 28]}

                                    "road22-3" {:path (js/paper.Path. "m 450.17896,457.74572 26.7203,-146.67121 c 0,0 32.70273,8.7046 36.88562,-15.68365 2.67206,-15.5794 49.95535,-294.79460861 49.95535,-294.79460861")                                                :light [31 33]}
                                    "road22-4" {:path (js/paper.Path. "m 858.53484,323.27291 c 0,0 -253.97818,15.90967 -325.53121,-10.82273 -24.63702,-9.20445 -20.64174,-6.97301 -16.28517,-33.69331 9.08236,-55.70516 46.15045,-279.03193248 46.15045,-279.03193248") :light [31 33]}

                                    "road21-3" {:path (js/paper.Path. "m 555.5254,-1.8681971 c 0,0 -47.28329,279.2152071 -49.95535,294.7946071 -4.18289,24.38825 -36.88562,15.68365 -36.88562,15.68365 l -26.7203,146.67121")                                    :light [34 32]}
                                    "road21-4" {:path (js/paper.Path. "m 553.42186,-1.5072867 c 0,0 -34.1929,219.6301067 -43.27526,275.3352667 -4.35657,26.7203 -17.52913,33.73172 7.84182,40.66132 120.43126,32.89352 340.95716,15.76622 340.95716,15.76622") :light [34 32]}
                                       }
                    :traffic-lights {
                                     0  {:point (js/paper.Path.Circle. (js/paper.Point.  372.95322  348.94907          ) 4)}
                                     1  {:point (js/paper.Path.Circle. (js/paper.Point.  372.54248  194.71568          ) 4)}
                                     2  {:point (js/paper.Path.Circle. (js/paper.Point.  369.05115  212.37755          ) 4)}
                                     3  {:point (js/paper.Path.Circle. (js/paper.Point.  367.35428  221.35835          ) 4)}
                                     4  {:point (js/paper.Path.Circle. (js/paper.Point.  363.87134  240.92763          ) 4)}
                                     5  {:point (js/paper.Path.Circle. (js/paper.Point.  423.16824  274.47931          ) 4)}
                                     6  {:point (js/paper.Path.Circle. (js/paper.Point.  438.06061  276.71777          ) 4)}
                                     7  {:point (js/paper.Path.Circle. (js/paper.Point.  451.05026  277.96457          ) 4)}
                                     8  {:point (js/paper.Path.Circle. (js/paper.Point.  483.03189  202.72513          ) 4)}
                                     9  {:point (js/paper.Path.Circle. (js/paper.Point.  484.26413  187.52769          ) 4)}
                                     10 {:point (js/paper.Path.Circle. (js/paper.Point.  486.31784  176.02693          ) 4)}
                                     11 {:point (js/paper.Path.Circle. (js/paper.Point.  488.78229  160.65938          ) 4)}
                                     12 {:point (js/paper.Path.Circle. (js/paper.Point.  431.48383  128.17557          ) 4)}
                                     13 {:point (js/paper.Path.Circle. (js/paper.Point.  419.77771  125.71112          ) 4)}
                                     14 {:point (js/paper.Path.Circle. (js/paper.Point.  406.83936  123.86277          ) 4)}
                                     15 {:point (js/paper.Path.Circle. (js/paper.Point.  362.90231  249.06598          ) 4)}

                                     17 {:point (js/paper.Path.Circle. (js/paper.Point.  348.00064    143.06494        ) 4) }
                                     19 {:point (js/paper.Path.Circle. (js/paper.Point.  372.13171    280.53378        ) 4) }

                                     21 {:point (js/paper.Path.Circle. (js/paper.Point.  530.33984    149.01009        ) 4) }
                                     20 {:point (js/paper.Path.Circle. (js/paper.Point.  461.2626    290.62381        ) 4) }
                                     22 {:point (js/paper.Path.Circle. (js/paper.Point.  516.1084    274.76978        ) 4) }
                                     23 {:point (js/paper.Path.Circle. (js/paper.Point.  341.22342    141.83272        ) 4) }

                                     25  {:point (js/paper.Path.Circle. (js/paper.Point.  329.64716  253.27734          ) 4)}
                                     24  {:point (js/paper.Path.Circle. (js/paper.Point.  344.16501  168.22285          ) 4)}

                                     26  {:point (js/paper.Path.Circle. (js/paper.Point.  332.28979  186.0901          ) 4)}

                                     27 {:point (js/paper.Path.Circle. (js/paper.Point.  372.13171    274.53378        ) 4) }
                                     29 {:point (js/paper.Path.Circle. (js/paper.Point.  412.85767    287.98468        ) 4) }
                                     30 {:point (js/paper.Path.Circle. (js/paper.Point.  459.82504    296.37418        ) 4) }
                                     28 {:point (js/paper.Path.Circle. (js/paper.Point.  399.03528    283.43582        ) 4) }

                                     31 {:point (js/paper.Path.Circle. (js/paper.Point.  510.14084    272.96191        ) 4) }
                                     32 {:point (js/paper.Path.Circle. (js/paper.Point.  524.88196    230.26973        ) 4) }
                                     33 {:point (js/paper.Path.Circle. (js/paper.Point.  528.05237    216.27959        ) 4) }
                                     34 {:point (js/paper.Path.Circle. (js/paper.Point.  535.71295    149.88141        ) 4) }
                                     }
;0 {:point (js/paper.Path.Circle. (js/paper.Point. 0 0) 4) :chan (chan (dropping-buffer 1))}
                    :sensors {
                              0  {:point (js/paper.Path.Circle. (js/paper.Point.  363.9169    334.16238        ) 4) :chan (chan (dropping-buffer 1))}
                              1  {:point (js/paper.Path.Circle. (js/paper.Point.  296.34995    187.11697        ) 4) :chan (chan (dropping-buffer 4))}
                              2  {:point (js/paper.Path.Circle. (js/paper.Point.  294.29623    199.64458        ) 4) :chan (chan (dropping-buffer 4))}
                              3  {:point (js/paper.Path.Circle. (js/paper.Point.  292.85864    209.50237        ) 4) :chan (chan (dropping-buffer 4))}
                              4  {:point (js/paper.Path.Circle. (js/paper.Point.  290.80493    224.49443        ) 4) :chan (chan (dropping-buffer 4))}
                              5  {:point (js/paper.Path.Circle. (js/paper.Point.  417.64990    323.27292        ) 4) :chan (chan (dropping-buffer 4))}
                              6  {:point (js/paper.Path.Circle. (js/paper.Point.  430.86774    325.12607        ) 4) :chan (chan (dropping-buffer 4))}
                              7  {:point (js/paper.Path.Circle. (js/paper.Point.  441.75626    326.75818        ) 4) :chan (chan (dropping-buffer 4))}
                              8  {:point (js/paper.Path.Circle. (js/paper.Point.  545.45721    211.53934        ) 4) :chan (chan (dropping-buffer 4))}
                              9  {:point (js/paper.Path.Circle. (js/paper.Point.  547.47571    197.51324        ) 4) :chan (chan (dropping-buffer 4))}
                              10 {:point (js/paper.Path.Circle. (js/paper.Point.  549.09802    185.60529        ) 4) :chan (chan (dropping-buffer 4))}
                              11 {:point (js/paper.Path.Circle. (js/paper.Point.  550.67340    172.06790        ) 4) :chan (chan (dropping-buffer 4))}
                              16 {:point (js/paper.Path.Circle. (js/paper.Point.  551.83112    162.88322        ) 4) :chan (chan (dropping-buffer 4))}
                              12 {:point (js/paper.Path.Circle. (js/paper.Point.  432.92145    117.08555        ) 4) :chan (chan (dropping-buffer 4))}
                              13 {:point (js/paper.Path.Circle. (js/paper.Point.  421.21533    115.44258        ) 4) :chan (chan (dropping-buffer 4))}
                              14 {:point (js/paper.Path.Circle. (js/paper.Point.  410.33066    113.59425        ) 4) :chan (chan (dropping-buffer 4))}
                              15 {:point (js/paper.Path.Circle. (js/paper.Point.  290.18881    237.12473        ) 4) :chan (chan (dropping-buffer 4))}

                              17 {:point (js/paper.Path.Circle. (js/paper.Point.  344.71475    134.64476        ) 4) :chan (chan (dropping-buffer 4))}
                              19 {:point (js/paper.Path.Circle. (js/paper.Point.  362.90234    278.40021        ) 4) :chan (chan (dropping-buffer 4))}
                              20 {:point (js/paper.Path.Circle. (js/paper.Point.  469.47742    294.32047        ) 4) :chan (chan (dropping-buffer 4))}
                              21 {:point (js/paper.Path.Circle. (js/paper.Point.  531.21118    142.62045        ) 4) :chan (chan (dropping-buffer 4))}
                              22 {:point (js/paper.Path.Circle. (js/paper.Point.  515.23706    283.77332        ) 4) :chan (chan (dropping-buffer 4))}

                              25 {:point (js/paper.Path.Circle. (js/paper.Point.  326.78836    261.55911        ) 4) :chan (chan (dropping-buffer 4))}
                              24 {:point (js/paper.Path.Circle. (js/paper.Point.  344.16501    171.71416        ) 4) :chan (chan (dropping-buffer 4))}
                              ;duplicate of 17
                              23 {:point (js/paper.Path.Circle. (js/paper.Point.  344.71475    134.64476        ) 4) :chan (chan (dropping-buffer 4))}
                              ;current
                              26 {:point (js/paper.Path.Circle. (js/paper.Point.  332.90591    181.9827        ) 4) :chan (chan (dropping-buffer 4))}
                              ;dup
                              27 {:point (js/paper.Path.Circle. (js/paper.Point.  362.90234    278.40021        ) 4) :chan (chan (dropping-buffer 4))}
                              28 {:point (js/paper.Path.Circle. (js/paper.Point.  403.55344    284.66806        ) 4) :chan (chan (dropping-buffer 4))}
                              29 {:point (js/paper.Path.Circle. (js/paper.Point.  406.17758    286.82294        ) 4) :chan (chan (dropping-buffer 4))}
                              30 {:point (js/paper.Path.Circle. (js/paper.Point.  469.47742    294.32047        ) 4) :chan (chan (dropping-buffer 4))}

                              31 {:point (js/paper.Path.Circle. (js/paper.Point.  515.23706    283.77332        ) 4) :chan (chan (dropping-buffer 4))}
                              32 {:point (js/paper.Path.Circle. (js/paper.Point.  525.55823    226.69336        ) 4) :chan (chan (dropping-buffer 4))}
                              33 {:point (js/paper.Path.Circle. (js/paper.Point.  527    221.20851        ) 4) :chan (chan (dropping-buffer 4))}
                              34 {:point (js/paper.Path.Circle. (js/paper.Point.  531.21118    142.62045        ) 4) :chan (chan (dropping-buffer 4))}
                   }})))



(def ui-state (r/atom {:speed 3 :sensor-refresh 300 :last-packed "last-packet" :connect-ip "127.0.0.1" :connect-port 9990 :display-sensors false :display-paths false :spawn-rate 0.6}))

(def cars (atom []))
(def lights (atom [0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0]))
(def cars-location-ahead (atom {}))

(defn reset-light-states! [ls]
  (reset! lights (reduce #(assoc %1 (.-id %2) (.-status %2)) @lights ls)))

;(add-car-channel! {:test "a"})
(defn add-car-channel! [car]
  (let [new-car (assoc car :chan (chan 10))]
    (swap! cars conj new-car)
    new-car))

